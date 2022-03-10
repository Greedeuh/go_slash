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
  },
};
