import { wrap } from '../../shared/js/dom'

export function init() {
  // Add wrappers around elements in the abstract to be able to use
  // Bootstrap's `panel` class
  const abstract = ':--Article > [data-itemprop="description"]'
  wrap(abstract, 'h2', 'div.panel-heading')
  wrap(abstract, 'p', 'div.panel-body')
}
