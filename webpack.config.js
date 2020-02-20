const globby = require('globby')
const path = require('path')

const MiniCssExtractPlugin = require('mini-css-extract-plugin')
const FileManagerPlugin = require('filemanager-webpack-plugin')
const { CleanWebpackPlugin } = require('clean-webpack-plugin')

// Convert absolute filepaths to project relative ones to use as
// output destinations.
const makeRelativePath = filepath =>
  path.relative(path.join(__dirname, contentSource), filepath)

// Strip `/src` from output destination pathnames.
// Otherwise Webpack outputs files at `/dist/src/*`
const fileLoaderOutputPath = (url, resourcePath, context) => {
  const relativePath = path
    .relative(context, resourcePath)
    .replace(`${contentSource}/`, '')

  return `${relativePath}`
}

module.exports = (env = {}, { mode }) => {
  const isDocs = env.docs === 'true'
  const isDevelopment = mode === 'development'

  const contentSource = 'src'
  const contentBase = isDocs ? 'docs' : 'dist'

  const entries = [
    './src/**/*.{css,ts,html,ttf,woff,woff2}',
    // Don’t compile test files for package distribution
    '!**/*.{d,test}.ts',
    // These files make use of Node APIs, and cannot easily be packaged for Browser targets
    '!**/scripts/{examples,extensions,selectors,themes}.ts',
    '!**/extensions/math/update.ts',
    // Don’t build HTML demo files for package distribution
    ...(isDocs || isDevelopment
      ? []
      : ['!**/*.html', '!**/demo/*', '!**/examples/*.html'])
  ]

  const entry = globby.sync(entries).reduce(
    (files, file) => ({
      ...files,
      [makeRelativePath(file)
        .replace(/.ts$/, '')
        .replace(/.css$/, '')]: file
    }),
    {}
  )

  return {
    entry,
    resolve: {
      extensions: ['.ts', '.tsx', '.js', '.css', '.html']
    },
    mode: mode || 'development',
    output: {
      path: path.resolve(__dirname, contentBase),
      filename: '[name].js'
    },
    devServer: {
      contentBase: `./${contentBase}`
    },
    plugins: [
      new CleanWebpackPlugin(),
      new MiniCssExtractPlugin(),
      // After a successful build, delete empty artifacts generated by Webpack for
      // non TypeScript/JavaScript files (i.e. for font and CSS files).
      new FileManagerPlugin({
        onEnd: {
          delete: [
            `${contentBase}/**/styles.js`,
            `${contentBase}/fonts/**/*.js`,
            `${contentBase}/generate`,
            ...(isDocs ? [] : [`${contentBase}/demo/`, `${contentBase}/share/`])
          ]
        }
      })
    ],
    module: {
      rules: [
        {
          test: /\.tsx?$/,
          use: {
            loader: 'ts-loader',
            options: {
              experimentalWatchApi: true
            }
          }
        },
        {
          test: /\.html$/i,
          use: [
            {
              loader: 'file-loader',
              options: {
                name: '[name].[ext]',
                outputPath: fileLoaderOutputPath
              }
            },
            'extract-loader',
            'html-loader'
          ]
        },
        {
          test: /\.(css)$/,
          use: [
            {
              loader: MiniCssExtractPlugin.loader,
              options: { hmr: isDevelopment, reloadAll: true }
            },
            {
              loader: 'css-loader',
              options: { importLoaders: 1, url: false, import: true }
            },
            'postcss-loader'
          ]
        },
        {
          test: /\.(eot|woff|woff2|svg|ttf|jpe?g|png|gif)$|html\.media\/.*$/,
          use: [
            {
              loader: 'file-loader',
              options: {
                name: '[folder]/[name].[ext]',
                outputPath: fileLoaderOutputPath
              }
            }
          ]
        }
      ]
    }
  }
}
