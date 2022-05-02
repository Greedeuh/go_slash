<template>
  <div role="list" class="list-group">
    <a
      v-for="user in users"
      :key="user.mail"
      role="listitem"
      class="list-group-item-action list-group-item d-flex justify-content-between align-items-start"
    >
      {{ user.mail }}
    </a>
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
}

export default defineComponent({
  name: "Partial",
  data(): Data {
    return {
      users: CONTEXT.users,
    };
  },
});
</script>

<style scoped></style>
