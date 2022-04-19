<template>
  <div
    class="modal fade"
    id="create_modal"
    tabindex="-1"
    role="dialog"
    aria-hidden="true"
  >
    <div class="modal-dialog">
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">Create team</h5>
          <button
            type="button"
            class="btn-close"
            data-bs-dismiss="modal"
            aria-label="Close"
          ></button>
        </div>
        <CreateTeamForm v-if="!success" @create="create" />
        <CreateTeamSuccess v-if="success" />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import axios from "axios";
import { defineComponent } from "vue";
import CreateTeamForm from "./CreateTeamForm.vue";
import CreateTeamSuccess from "./CreateTeamSuccess.vue";

export default defineComponent({
  name: "CreateTeamModal",
  components: { CreateTeamForm, CreateTeamSuccess },
  data() {
    return { success: false };
  },

  methods: {
    create(team: { slug: string; title: string; is_private: string }) {
      axios.post("/go/teams", team).then((res) => {
        if (res.status === 201) {
          this.success = true;
        }
      });
    },
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
