<template>
  <div role="list" class="accordion">
    <div
      v-for="(user, index) in users"
      :key="user.mail"
      role="listitem"
      class="accordion-item"
    >
      <h2 class="accordion-header">
        <button
          class="accordion-button"
          type="button"
          data-bs-toggle="collapse"
          :data-bs-target="'#collapse' + index"
          aria-expanded="false"
        >
          {{ user.mail }}
        </button>
      </h2>
      <div :id="'collapse' + index" class="accordion-collapse collapse">
        <div class="accordion-body">
          <strong>Capabilities :</strong>
          <div
            v-for="capability in capabilities"
            :key="capability"
            class="form-check form-switch"
          >
            <input
              class="form-check-input"
              v-model="is_private"
              :name="capability"
              type="checkbox"
              role="switch"
              :checked="user.capabilities.includes(capability)"
            />
            <label class="form-check-label">
              {{ capability }}
            </label>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import { User } from "../../models";

interface Window {
  context: {
    user: User;
    users: User[];
  };
}

let win = window as unknown as Window;
const CONTEXT = win.context;

interface Data {
  users: User[];
  capabilities: string[];
}

const ALL_CAPABILITIES = [
  "Features",
  "ShortcutsWrite",
  "TeamsRead",
  "TeamsWrite",
  "TeamsWriteWithValidation",
  "UsersAdmin",
  "UsersTeamsRead",
  "UsersTeamsWrite",
].sort();

export default defineComponent({
  name: "Partial",
  data(): Data {
    return {
      users: CONTEXT.users,
      capabilities: ALL_CAPABILITIES,
    };
  },
});
</script>

<style scoped></style>
