<template>
  <div aria-label="Create team result">
    <div class="modal-body">
      Success !
      <span v-if="capabilities.includes('TeamsCreateWithValidation')"
        >Your Admins will now have to validate your team.</span
      >
    </div>
    <div class="modal-footer">
      <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">
        Close
      </button>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "CreateTeamSuccess",
  props: {
    capabilities: Array,
  },
});
</script>

<style scoped>
span {
  line-height: 38px;
}

button {
  margin-left: 0.5em;
}

.list-group {
  margin-bottom: 16px;
}
</style>
