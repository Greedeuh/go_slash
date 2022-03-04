module.exports = {
  outputDir: "../templates/vue_dist",
  assetsDir: "../../public/vue",
  pages: {
    search: {
      entry: "src/partials/search/main.ts",
      template: "public/search.html",
      filename: "search.html.hbs",
    },
  },
};
