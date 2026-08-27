import { CaptionPlugin } from "@platejs/caption/react"
import {
  BlockquotePlugin,
  BoldPlugin,
  CodePlugin,
  H1Plugin,
  H2Plugin,
  H3Plugin,
  HorizontalRulePlugin,
  ItalicPlugin,
  StrikethroughPlugin,
  SubscriptPlugin,
  SuperscriptPlugin,
  UnderlinePlugin,
} from "@platejs/basic-nodes/react"
import {
  FontBackgroundColorPlugin,
  FontColorPlugin,
  FontFamilyPlugin,
  FontSizePlugin,
} from "@platejs/basic-styles/react"
import {
  CodeBlockPlugin,
  CodeLinePlugin,
  CodeSyntaxPlugin,
} from "@platejs/code-block/react"
import { IndentPlugin } from "@platejs/indent/react"
import { LinkPlugin } from "@platejs/link/react"
import { ListPlugin } from "@platejs/list/react"
import { ImagePlugin } from "@platejs/media/react"
import {
  TableCellHeaderPlugin,
  TableCellPlugin,
  TablePlugin,
  TableRowPlugin,
} from "@platejs/table/react"
import { createLowlight } from "lowlight"
import { KEYS } from "platejs"
import { ParagraphPlugin } from "platejs/react"

import bash from "highlight.js/lib/languages/bash"
import c from "highlight.js/lib/languages/c"
import cpp from "highlight.js/lib/languages/cpp"
import css from "highlight.js/lib/languages/css"
import go from "highlight.js/lib/languages/go"
import java from "highlight.js/lib/languages/java"
import javascript from "highlight.js/lib/languages/javascript"
import json from "highlight.js/lib/languages/json"
import markdown from "highlight.js/lib/languages/markdown"
import python from "highlight.js/lib/languages/python"
import rust from "highlight.js/lib/languages/rust"
import sql from "highlight.js/lib/languages/sql"
import typescript from "highlight.js/lib/languages/typescript"
import xml from "highlight.js/lib/languages/xml"
import yaml from "highlight.js/lib/languages/yaml"

import {
  BlockquoteElement,
  CodeBlockElement,
  CodeLeaf,
  CodeLineElement,
  CodeSyntaxLeaf,
  H1Element,
  H2Element,
  H3Element,
  HrElement,
  ImageElement,
  LinkElement,
  ParagraphElement,
  TableCellElement,
  TableCellHeaderElement,
  TableElement,
  TableRowElement,
} from "./nodes"

/** The fifteen grammars the toolbar's code block offers. */
export const codeBlockLanguages = {
  bash: "Bash",
  c: "C",
  cpp: "C++",
  css: "CSS",
  go: "Go",
  java: "Java",
  javascript: "JavaScript",
  json: "JSON",
  markdown: "Markdown",
  python: "Python",
  rust: "Rust",
  sql: "SQL",
  typescript: "TypeScript",
  xml: "HTML/XML",
  yaml: "YAML",
} as const

const lowlight = createLowlight({
  bash,
  c,
  cpp,
  css,
  go,
  java,
  javascript,
  json,
  markdown,
  python,
  rust,
  sql,
  typescript,
  xml,
  yaml,
})

export const richTextPlugins = [
  ParagraphPlugin.withComponent(ParagraphElement),
  H1Plugin.withComponent(H1Element),
  H2Plugin.withComponent(H2Element),
  H3Plugin.withComponent(H3Element),
  BlockquotePlugin.withComponent(BlockquoteElement),
  HorizontalRulePlugin.withComponent(HrElement),

  BoldPlugin,
  ItalicPlugin,
  UnderlinePlugin,
  StrikethroughPlugin,
  SubscriptPlugin,
  SuperscriptPlugin,
  CodePlugin.withComponent(CodeLeaf),

  FontFamilyPlugin,
  FontSizePlugin,
  FontColorPlugin,
  FontBackgroundColorPlugin,

  IndentPlugin,
  ListPlugin,

  CodeBlockPlugin.configure({
    options: { lowlight },
  }).withComponent(CodeBlockElement),
  CodeLinePlugin.withComponent(CodeLineElement),
  CodeSyntaxPlugin.withComponent(CodeSyntaxLeaf),

  LinkPlugin.withComponent(LinkElement),
  ImagePlugin.withComponent(ImageElement),
  CaptionPlugin.configure({
    options: { query: { allow: [KEYS.img] } },
  }),

  TablePlugin.withComponent(TableElement),
  TableRowPlugin.withComponent(TableRowElement),
  TableCellPlugin.withComponent(TableCellElement),
  TableCellHeaderPlugin.withComponent(TableCellHeaderElement),
]
