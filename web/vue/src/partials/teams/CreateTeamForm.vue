<template>
  <form @submit.prevent="create">
    <div class="modal-body">
      <div class="input-group mb-3">
        <label class="input-group-text" for="slug">Slug</label>
        <input
          required
          type="text"
          v-model="slug"
          name="slug"
          id="slug"
          class="form-control"
        />
      </div>
      <div class="input-group mb-3">
        <label class="input-group-text" for="title">Title</label>
        <input
          required
          type="text"
          v-model="title"
          name="title"
          id="title"
          class="form-control"
        />
      </div>
      <div class="form-check form-switch">
        <input
          class="form-check-input"
          v-model="is_private"
          name="is_private"
          type="checkbox"
          id="checkbox"
          role="switch"
        />
        <label class="form-check-label" for="checkbox"
          >Private</label
        >
      </div>
    </div>
    <div class="modal-footer">
      <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">
        Close
      </button>
      <button type="submit" class="btn btn-primary" aria-label="Create team">
        Create
      </button>
    </div>
  </form>
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "CreateTeamForm",
  data() {
    return { slug: "", title: "", is_private: false };
  },
  emits: ["create"],
  methods: {
    create() {
      this.$emit("create", {
        slug: this.slug,
        title: this.title,
        is_private: this.is_private,
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
