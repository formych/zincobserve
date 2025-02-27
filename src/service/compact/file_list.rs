// Copyright 2022 Zinc Labs Inc. and Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use ahash::AHashMap as HashMap;
use bytes::Buf;
use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use std::io::{BufRead, BufReader, Write};

use crate::common::json;
use crate::infra::config::{CONFIG, STREAM_SCHEMAS};
use crate::infra::db::etcd;
use crate::infra::ider;
use crate::infra::storage;
use crate::meta::common::FileKey;
use crate::service::db;

/// check all streams done compact in this hour
/// merge all small file list keys in this hour to a single file and upload to storage
/// delete all small file list keys in this hour from storage
/// node should load new file list from storage
pub async fn run(offset: i64) -> Result<(), anyhow::Error> {
    let mut offset = offset;
    if offset == 0 {
        // get earilest date from schema
        offset = Utc::now().timestamp_micros();
        for item in STREAM_SCHEMAS.iter() {
            let schema = item.value();
            if let Some(val) = schema.first().unwrap().metadata.get("created_at") {
                let time_min = val.parse().unwrap();
                if time_min < offset {
                    offset = time_min;
                }
            }
        }
    }

    // still not found, just return
    if offset == 0 {
        return Ok(()); // no stream
    }
    let offset_time: DateTime<Utc> = Utc.timestamp_nanos(offset * 1000);
    let offset_time_hour = Utc
        .with_ymd_and_hms(
            offset_time.year(),
            offset_time.month(),
            offset_time.day(),
            offset_time.hour(),
            0,
            0,
        )
        .unwrap()
        .timestamp_micros();

    // check compact is done
    let offsets = db::compact::files::list_offset().await?;
    if offsets.is_empty() {
        return Ok(()); // no stream
    }
    // compact offset already is next hour, we need fix it, get the latest compact offset
    for (_, val) in offsets {
        let val = val - Duration::hours(1).num_microseconds().unwrap();
        if val < offset {
            return Ok(()); // compact is not done
        }
    }

    // output file list
    merge_file_list(offset).await?;

    // write new sync offset
    offset = offset_time_hour + Duration::hours(1).num_microseconds().unwrap();
    db::compact::file_list::set_offset(offset).await
}

/// merge and delete the small file list keys in this hour from etcd
/// upload new file list into storage
async fn merge_file_list(offset: i64) -> Result<(), anyhow::Error> {
    let mut locker = None;
    if !CONFIG.common.local_mode {
        // get a cluster lock for merge file list
        let lock_key = "compactor/file_list";
        let mut lock = etcd::Locker::new(lock_key);
        if lock.lock(CONFIG.etcd.command_timeout).await.is_err() {
            return Ok(()); // lock failed, just skip
        }
        locker = Some(lock);
    }

    // get all small file list keys in this hour
    let offset = Utc.timestamp_nanos(offset * 1000);
    let offset_prefix = offset.format("/%Y/%m/%d/%H/").to_string();
    let key = format!("file_list{}", offset_prefix);
    let storage = &storage::DEFAULT;
    let file_list = storage.list(&key).await?;
    if file_list.len() <= 1 {
        if locker.is_some() {
            // release cluster lock
            let mut lock = locker.unwrap();
            lock.unlock().await?;
        }
        return Ok(()); // only one file list, no need merge
    }

    // filter deleted file keys
    let mut filter_file_keys: HashMap<String, FileKey> = HashMap::with_capacity(1024);
    for file in file_list.clone() {
        log::info!("[COMPACT] merge small file list: {}", file);
        let data = storage.get(&file).await?;
        // uncompress file
        let uncompress = zstd::decode_all(data.reader())?;
        let uncompress_reader = BufReader::new(uncompress.reader());
        // parse file list
        for line in uncompress_reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            let item: FileKey = json::from_slice(line.as_bytes())?;
            match filter_file_keys.get(&item.key) {
                Some(_) => {
                    if item.deleted {
                        filter_file_keys.insert(item.key.clone(), item);
                    }
                }
                None => {
                    filter_file_keys.insert(item.key.clone(), item);
                }
            }
        }
    }

    // write new file list
    let id = ider::generate();
    let file_name = format!("file_list{}{}.json.zst", offset_prefix, id);
    let mut buf = zstd::Encoder::new(Vec::new(), 3)?;
    for (_, item) in filter_file_keys.iter() {
        if item.deleted {
            continue;
        }
        let val = json::to_vec(&item)?;
        buf.write_all(val.as_slice())?;
        buf.write_all(b"\n")?;
    }
    let compressed_bytes = buf.finish().unwrap();

    match storage.put(&file_name, compressed_bytes.into()).await {
        Ok(_) => {
            log::info!("[COMPACT] merge file list success, new file: {}", file_name);
            // delete all small file list keys in this hour from storage
            for file in file_list {
                storage.del(&file).await?;
            }
        }
        Err(err) => {
            log::error!("[COMPACT] upload file list failed: {}", err);
        }
    }

    if locker.is_some() {
        // release cluster lock
        let mut lock = locker.unwrap();
        lock.unlock().await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[actix_web::test]
    async fn test_compact() {
        let off_set = Duration::hours(2).num_microseconds().unwrap();
        let _ = db::compact::files::set_offset("nexus", "default", "logs".into(), off_set).await;
        let off_set_for_run = Duration::hours(1).num_microseconds().unwrap();
        let resp = run(off_set_for_run).await;
        assert!(resp.is_ok());
    }
}
