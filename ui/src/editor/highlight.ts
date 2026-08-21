import { HighlightStyle } from '@codemirror/language'
import { tags } from '@lezer/highlight'

/** CSS classes applied to Markdown tokens in the source editor. */
export const markdownTokenClasses = [
  'cm-md-heading',
  'cm-md-em',
  'cm-md-strong',
  'cm-md-strike',
  'cm-md-link',
  'cm-md-code',
  'cm-md-quote',
  'cm-md-comment',
  'cm-md-mark',
] as const

/** Class-based Markdown highlighting; colors come from theme CSS variables. */
export const markdownHighlightStyle = HighlightStyle.define([
  { tag: tags.heading1, class: 'cm-md-heading' },
  { tag: tags.heading2, class: 'cm-md-heading' },
  { tag: tags.heading3, class: 'cm-md-heading' },
  { tag: tags.heading4, class: 'cm-md-heading' },
  { tag: tags.heading5, class: 'cm-md-heading' },
  { tag: tags.heading6, class: 'cm-md-heading' },
  { tag: tags.heading, class: 'cm-md-heading' },
  { tag: tags.emphasis, class: 'cm-md-em' },
  { tag: tags.strong, class: 'cm-md-strong' },
  { tag: tags.strikethrough, class: 'cm-md-strike' },
  { tag: tags.link, class: 'cm-md-link' },
  { tag: tags.labelName, class: 'cm-md-link' },
  { tag: tags.monospace, class: 'cm-md-code' },
  { tag: tags.quote, class: 'cm-md-quote' },
  { tag: tags.comment, class: 'cm-md-comment' },
  { tag: tags.url, class: 'cm-md-mark' },
  { tag: tags.processingInstruction, class: 'cm-md-mark' },
  { tag: tags.meta, class: 'cm-md-mark' },
  { tag: tags.punctuation, class: 'cm-md-mark' },
  { tag: tags.escape, class: 'cm-md-mark' },
  { tag: tags.list, class: 'cm-md-mark' },
  { tag: tags.contentSeparator, class: 'cm-md-mark' },
  { tag: tags.atom, class: 'cm-md-mark' },
  { tag: tags.character, class: 'cm-md-mark' },
  { tag: tags.string, class: 'cm-md-mark' },
])
