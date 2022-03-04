module.exports = {
  outputDir: "../templates/vue_dist",
  assetsDir: "../../public/vue",
  pages: {
    search: {
      entry: "src/pages/search/main.js",
      template: "public/search.html",
      filename: "search.html.hbs",
    },
  },
};
