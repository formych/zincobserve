<!-- Copyright 2022 Zinc Labs Inc. and Contributors

 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at

     http:www.apache.org/licenses/LICENSE-2.0

 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License. 
-->

<!-- eslint-disable vue/x-invalid-end-tag -->
<template>
  <q-page class="ingestionPage">
    <div class="head q-table__title q-pb-md">
      {{ t("ingestion.header") }}

      <q-btn
        class="q-ml-md q-mb-xs text-bold no-border right float-right"
        padding="sm lg"
        color="secondary"
        no-caps
        icon="lock_reset"
        :label="t(`ingestion.passwordLabel`)"
        @click="showUpdateDialogFn"
      />
      <ConfirmDialog
        title="Reset Token"
        message="Are you sure you want to update token for this organization?"
        @update:ok="updatePasscode"
        @update:cancel="confirmUpdate = false"
        v-model="confirmUpdate"
      />
    </div>
    <q-separator class="separator" />
    <q-splitter
      v-model="splitterModel"
      unit="px"
      style="min-height: calc(100vh - 130px)"
    >
      <template v-slot:before>
        <q-tabs
          v-model="ingestiontab"
          indicator-color="transparent"
          class="text-secondary"
          inline-label
          vertical
        >
          <q-route-tab
            default
            name="curl"
            :to="'/ingestion/curl'"
            icon="data_object"
            label="Curl"
            content-class="tab_content"
          />
          <q-route-tab
            default
            name="fluentbit"
            :to="'/ingestion/fluentbit'"
            icon="img:/src/assets/images/ingestion/fluentbit_icon.png"
            label="FluentBit"
            content-class="tab_content"
          />
          <q-route-tab
            name="fluentd"
            :to="'/ingestion/fluentd'"
            icon="img:/src/assets/images/ingestion/fluentd_icon.svg"
            label="Fluentd"
            content-class="tab_content"
          />
          <q-route-tab
            name="vector"
            :to="'/ingestion/vector'"
            icon="img:/src/assets/images/ingestion/vector_icon.png"
            label="Vector"
            content-class="tab_content"
          />
        </q-tabs>
      </template>

      <template v-slot:after>
        <q-tab-panels
          v-model="ingestiontab"
          animated
          swipeable
          vertical
          transition-prev="jump-up"
          transition-next="jump-up"
        >
          <q-tab-panel name="curl">
            <router-view
              title="CURL"
              :currOrgIdentifier="currentOrgIdentifier"
              :currUserEmail="currentUserEmail"
              @copy-to-clipboard-fn="copyToClipboardFn"
            >
            </router-view>
          </q-tab-panel>
          <q-tab-panel name="fluentbit">
            <router-view
              title="Fluent Bit"
              :currOrgIdentifier="currentOrgIdentifier"
              :currUserEmail="currentUserEmail"
              @copy-to-clipboard-fn="copyToClipboardFn"
            >
            </router-view>
          </q-tab-panel>

          <q-tab-panel name="fluentd">
            <router-view
              title="Fluentd"
              :currOrgIdentifier="currentOrgIdentifier"
              :currUserEmail="currentUserEmail"
              @copy-to-clipboard-fn="copyToClipboardFn"
            >
            </router-view>
          </q-tab-panel>

          <q-tab-panel name="vector">
            <router-view
              title="Vector"
              :currOrgIdentifier="currentOrgIdentifier"
              :currUserEmail="currentUserEmail"
              @copy-to-clipboard-fn="copyToClipboardFn"
            >
            </router-view>
          </q-tab-panel>
        </q-tab-panels>
      </template>
    </q-splitter>
  </q-page>
</template>

<script lang="ts">
// @ts-ignore
import { defineComponent, ref, onMounted, onActivated } from "vue";
import { useI18n } from "vue-i18n";
import { useStore } from "vuex";
import { useRouter } from "vue-router";
import { copyToClipboard, useQuasar } from "quasar";
import organizationsService from "../services/organizations";
// import { config } from "../constants/config";
import config from "../aws-exports";
import ConfirmDialog from "../components/ConfirmDialog.vue";
import segment from "../services/segment_analytics";

export default defineComponent({
  name: "PageIngestion",
  components: { ConfirmDialog },
  setup() {
    const { t } = useI18n();
    const store = useStore();
    const q = useQuasar();
    const router: any = useRouter();
    const ingestiontab = ref("fluentbit");
    const rowData: any = ref({});
    const confirmUpdate = ref<boolean>(false);
    const currentOrgIdentifier: any = ref(
      store.state.selectedOrganization.identifier
    );

    onMounted(() => {
      if (router.currentRoute.value.name == "ingestion") {
        ingestiontab.value = "curl";
        router.push({ path: "/ingestion/curl" });
      } else {
        ingestiontab.value = router.currentRoute.value.name;
        router.push({ path: "/ingestion/" + router.currentRoute.value.name });
      }
      getOrganizationPasscode();
    });

    onActivated(() => {
      if (router.currentRoute.value.name == "ingestion") {
        ingestiontab.value = "curl";
        router.push({ path: "/ingestion/curl" });
      } else {
        ingestiontab.value = router.currentRoute.value.name;
        router.push({ path: "/ingestion/" + router.currentRoute.value.name });
      }

      segment.track("Button Click", {
        button: router.currentRoute.value.name,
        user_org: store.state.selectedOrganization.identifier,
        user_id: store.state.userInfo.email,
        page: "Ingestion",
      });

      getOrganizationPasscode();
    });

    const getOrganizationPasscode = () => {
      organizationsService
        .get_organization_passcode(store.state.selectedOrganization.identifier)
        .then((res) => {
          if (res.data.data.token == "") {
            q.notify({
              type: "negative",
              message: "API Key not found.",
              timeout: 5000,
            });
          } else {
            store.dispatch("setOrganizationPasscode", res.data.data.passcode);
            currentOrgIdentifier.value =
              store.state.selectedOrganization.identifier;
          }
        });
    };

    const copyToClipboardFn = (content: any) => {
      copyToClipboard(content.innerText)
        .then(() => {
          q.notify({
            type: "positive",
            message: "Content Copied Successfully!",
            timeout: 5000,
          });
        })
        .catch(() => {
          q.notify({
            type: "negative",
            message: "Error while copy content.",
            timeout: 5000,
          });
        });

      segment.track("Button Click", {
        button: "Copy to Clipboard",
        ingestion: router.currentRoute.value.name,
        user_org: store.state.selectedOrganization.identifier,
        user_id: store.state.userInfo.email,
        page: "Ingestion",
      });
    };

    const updatePasscode = () => {
      organizationsService
        .update_organization_passcode(
          store.state.selectedOrganization.identifier
        )
        .then((res) => {
          if (res.data.data.token == "") {
            q.notify({
              type: "negative",
              message: "API Key not found.",
              timeout: 5000,
            });
          } else {
            q.notify({
              type: "positive",
              message: "Token reset successfully.",
              timeout: 5000,
            });
            store.dispatch("setOrganizationPasscode", res.data.data.passcode);
            currentOrgIdentifier.value =
              store.state.selectedOrganization.identifier;
          }
        })
        .catch((e) => {
          q.notify({
            type: "negative",
            message: "Error while updating Token." + e.error,
            timeout: 5000,
          });
        });

      segment.track("Button Click", {
        button: "Update Passcode",
        user_org: store.state.selectedOrganization.identifier,
        user_id: store.state.userInfo.email,
        page: "Ingestion",
      });
    };

    const showUpdateDialogFn = () => {
      confirmUpdate.value = true;
    };

    return {
      t,
      store,
      router,
      config,
      rowData,
      ingestiontab,
      splitterModel: ref(170),
      getOrganizationPasscode,
      currentUserEmail: store.state.userInfo.email,
      currentOrgIdentifier,
      copyToClipboardFn,
      updatePasscode,
      showUpdateDialogFn,
      confirmUpdate,
    };
  },
  computed: {
    selectedOrg() {
      return this.store.state.selectedOrganization.identifier;
    },
  },
  watch: {
    selectedOrg(newVal: any, oldVal: any) {
      if (
        newVal != oldVal &&
        (this.router.currentRoute.value.name == "ingestion" ||
          this.router.currentRoute.value.name == "fluentbit" ||
          this.router.currentRoute.value.name == "fluentd" ||
          this.router.currentRoute.value.name == "vector" ||
          this.router.currentRoute.value.name == "curl")
      ) {
        this.getOrganizationPasscode();
      }
    },
  },
});
</script>

<style scoped lang="scss">
.ingestionPage {
  padding: 1.5rem 1.5rem 0;
  .head {
    padding-bottom: 1rem;
  }
  .q-tabs {
    &--vertical {
      margin: 1.5rem 1rem 0 0;
      .q-tab {
        justify-content: flex-start;
        padding: 0 1rem 0 1.25rem;
        border-radius: 0.5rem;
        margin-bottom: 0.5rem;
        color: $dark;

        &__content.tab_content {
          .q-tab {
            &__icon + &__label {
              padding-left: 0.875rem;
              font-weight: 600;
            }
          }
        }
        &--active {
          background-color: $accent;
        }
      }
    }
  }
}
</style>
