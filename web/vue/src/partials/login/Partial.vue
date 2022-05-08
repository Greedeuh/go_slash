<template>
  <div>
    <div v-if="error" class="alert alert-danger" role="alert">
      Wrong credentials :/ !
    </div>
    <div v-if="success" class="alert alert-success" role="alert">
      Login success !
    </div>
    <form v-if="simple_salt" @submit.prevent="submit_simple_login" action="/">
      <div id="simple">
        <div class="input-group mb-3">
          <span class="input-group-text" id="basic-addon1">Mail</span>
          <input v-model="mail" type="email" class="form-control" required />
        </div>
        <div class="input-group mb-3">
          <span class="input-group-text" id="basic-addon1">Password</span>
          <input ref="pwd" type="password" class="form-control" required />
        </div>
        <input type="submit" class="btn btn-primary" value="Login" />
      </div>
    </form>
    <a href="/go/login/google" aria-label="Login with google"
      >Login with google</a
    >
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import axios from "axios";
import sha256 from "crypto-js/sha256";
import hex from "crypto-js/enc-hex";
import addMonths from "date-fns/addMonths";
import formatISO from "date-fns/formatISO";

interface Window {
  context: WindowContext;
}

interface WindowContext {
  simple_salt: string;
}

let win = window as unknown as Window;
const SIMPLE_SALT = win.context.simple_salt;

export default defineComponent({
  name: "Partial",
  data() {
    return {
      mail: "",
      simple_salt: SIMPLE_SALT,
      error: false,
      success: false,
    };
  },
  methods: {
    submit_simple_login() {
      axios
        .post("/go/login", {
          mail: this.mail,
          pwd: hex.stringify(
            sha256(
              (this.$refs as unknown as any).pwd.value + // eslint-disable-line
                this.simple_salt
            )
          ),
        })
        .then((res) => {
          if (res.status === 200) {
            this.success = true;
            this.error = false;

            document.cookie =
              "go_session_id=" +
              res.data.token +
              "; " +
              formatISO(addMonths(new Date(), 1)) +
              "; path=/";

            const params = new URLSearchParams(window.location.search);

            if (params.has("from")) {
              setTimeout(() => {
                window.location.href = params.get("from") as string;
              }, 500);
            } else {
              setTimeout(() => {
                window.location.pathname = "";
              }, 500);
            }
          } else {
            this.error = true;
          }
        })
        .catch((e) => {
          console.error(e);
          this.error = true;
        });
    },
  },
});
</script>

<style></style>
