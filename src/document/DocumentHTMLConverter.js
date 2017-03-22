import { DefaultDOMElement } from 'substance'

export default class DocumentHTMLConverter {

  /*
    Read a storer (source file layout) and store to a buffer (internal Stencila
    file format)

    Original fileName is needed because otherwise we don't know what to read
    from the storer.

    TODO: The source archive could include binaries, which we should also
          consider.
  */
  importDocument(storer, buffer, fileName) {
    let manifest = {
      "type": "document"
    }
    return storer.readFile(
      fileName,
      'text/html'
    ).then((html) => {
      manifest.title = this._extractTitle(html)
      return buffer.writeFile(
        'index.html',
        'text/html',
        html
      )
    }).then(() => {
      return buffer.writeFile(
        'stencila-manifest.json',
        'application/json',
        JSON.stringify(manifest, null, '  ')
      )
    }).then(() => {
      return manifest
    })
  }

  /*
    Takes a buffer and writes back to the storer
  */
  exportDocument(buffer, storer, fileName) {
    return buffer.readFile('index.html', 'text/html').then((htmlFile) => {
      return storer.writeFile(fileName, 'text/html', htmlFile)
    })
  }

  _extractTitle(html) {
    var htmlDoc = DefaultDOMElement.parseHTML(html)
    let titleEl = htmlDoc.find('div[data-title]')
    return titleEl ? titleEl.textContent : 'Untitled'
  }
}

DocumentHTMLConverter.match = function(fileName) {
  return fileName.indexOf('.html') >= 0
}
