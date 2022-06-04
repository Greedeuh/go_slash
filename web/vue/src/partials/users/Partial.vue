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
          class="accordion-button collapsed"
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
          <Capabilities
            :user_capabilities="user.capabilities"
            @toggle="(capability) => toggle(user, capability)"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import axios from "axios";
import { defineComponent } from "vue";
import { Capability, User } from "../../models";
import Capabilities from "../../components/Capabilities.vue";

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
}

export default defineComponent({
  name: "Partial",
  components: {
    Capabilities,
  },
  data(): Data {
    return {
      users: CONTEXT.users,
    };
  },
  methods: {
    toggle(
      user: User,
      { capability, value }: { capability: Capability; value: boolean }
    ) {
      if (value) {
        axios
          .put(`/go/users/${user.mail}/capabilities/${capability}`)
          .then((res) => {
            if (res.status === 200) {
              user.capabilities = [...user.capabilities, capability];
            }
          });
      } else {
        axios
          .delete(`/go/users/${user.mail}/capabilities/${capability}`)
          .then((res) => {
            if (res.status === 200) {
              user.capabilities = user.capabilities.filter(
                (c) => c !== capability
              );
            }
          });
      }
    },
  },
});
</script>

<style scoped></style>
