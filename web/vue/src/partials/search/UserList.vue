<template>
  <div role="list" class="accordion mb-4" aria-label="User list">
    <div
      v-for="(user_link, index) in user_links"
      :key="user_link.user_mail"
      role="listitem"
      class="accordion-item"
    >
      <h2 class="accordion-header">
        <button
          class="accordion-button collapsed"
          type="button"
          data-bs-toggle="collapse"
          :data-bs-target="'#collapse' + index"
          aria-expanded="false"
        >
          {{ user_link.user_mail }}
        </button>
      </h2>
      <div :id="'collapse' + index" class="accordion-collapse collapse">
        <div class="accordion-body">
          <button
            @click="kick(user_link)"
            type="button"
            class="btn btn-danger"
            aria-label="Kick user"
          >
            Kick
          </button>
          <button
            v-if="!user_link.is_accepted"
            @click="accept(user_link)"
            type="button"
            class="btn btn-success"
            aria-label="Accept candidature"
          >
            Accept candidature
          </button>
          <Capabilities
            :user_capabilities="user_link.capabilities"
            type="team"
            @toggle="(capability) => toggle(user_link, capability)"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from "vue";
import { UserTeamLink } from "../../models";
import Capabilities from "../../components/Capabilities.vue";

export default defineComponent({
  name: "UserList",
  components: { Capabilities },
  props: {
    user_links: { required: true, type: Array as PropType<UserTeamLink[]> },
  },
  emits: ["toggle", "kick", "accept"],
  methods: {
    toggle(user_link: UserTeamLink, capability: string) {
      this.$emit("toggle", user_link, capability);
    },
    kick(user_link: UserTeamLink) {
      this.$emit("kick", user_link);
    },
    accept(user_link: UserTeamLink) {
      this.$emit("accept", user_link);
    },
  },
});
</script>

<style></style>
