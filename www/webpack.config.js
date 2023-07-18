const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = [
  {
    entry: "./bootstrap.js",
    output: {
      path: path.resolve(__dirname, "dist"),
      filename: "bootstrap.js",
    },
    mode: "development",
    plugins: [
      new CopyWebpackPlugin(['public', 'index.html'])
    ],
  },
  {
    entry: "./share.js",
    output: {
      path: path.resolve(__dirname, "dist"),
      filename: "share/share.js",
    },
    mode: "development",
    plugins: [
      new CopyWebpackPlugin([{
        from: 'share.html',
        to: 'share/index.html',
      }])
    ],
  },
];
