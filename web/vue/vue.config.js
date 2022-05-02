module.exports = {
  outputDir: "../templates/vue_dist",
  assetsDir: "../../public/vue",
  pages: {
    search: {
      entry: "src/partials/search/main.ts",
      template: "public/index.html",
      filename: "search.html.hbs",
    },
    features: {
      entry: "src/partials/features/main.ts",
      template: "public/index.html",
      filename: "features.html.hbs",
    },
    login: {
      entry: "src/partials/login/main.ts",
      template: "public/index.html",
      filename: "login.html.hbs",
    },
    teams: {
      entry: "src/partials/teams/main.ts",
      template: "public/index.html",
      filename: "teams.html.hbs",
    },
    users: {
      entry: "src/partials/users/main.ts",
      template: "public/index.html",
      filename: "users.html.hbs",
    },
  },
};
