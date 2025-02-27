#!/bin/bash

# cargo llvm-cov >report.json && ls -l | grep TOTAL report.json | xargs > coverage.txt
summary="$(RUSTFLAGS='-C target-cpu=native' cargo llvm-cov --ignore-filename-regex job >report.json && ls -l | grep TOTAL report.json | xargs)"
echo "Coverage Summary $summary %"

region_cov="$(cut -d' ' -f4 <<<"$summary")"
region_cov=${region_cov//%/}
func_cov="$(cut -d' ' -f7 <<<"$summary")"
func_cov=${func_cov//%/}
line_cov="$(cut -d' ' -f10 <<<"$summary")"
line_cov=${line_cov//%/}

echo "region_cov $region_cov"
echo "func_cov $func_cov"
echo "line_cov $line_cov"

# enable threshold
#COVERAGE_THRESHOLD=80
FUNC_COV_THRESHOLD=40
LINE_COV_THRESHOLD=35
REGION_COV_THRESHOLD=25

# clean up
# find ./target -name llvm-cov-target -type d|xargs rm -fR
# clean up finished

func_diff=$(echo "$func_cov < $FUNC_COV_THRESHOLD" | bc)
line_diff=$(echo "$line_cov < $LINE_COV_THRESHOLD" | bc)
region_diff=$(echo "$region_cov < $REGION_COV_THRESHOLD" | bc)

if [ $func_diff -eq 1 ] || [ $line_diff -eq 1 ] || [ $region_diff -eq 1 ]; then
    echo "Coverage is below threshold of function coverage $FUNC_COV_THRESHOLD% or line coverage $LINE_COV_THRESHOLD% or region coverage $REGION_COV_THRESHOLD%"
    exit 1
else
    echo "Coverage is above threshold of function coverage $FUNC_COV_THRESHOLD% & line coverage $LINE_COV_THRESHOLD% & region coverage $REGION_COV_THRESHOLD%"
    exit 0
fi