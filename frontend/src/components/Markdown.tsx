"use client";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

export interface MarkdownProps { source: string }

export function Markdown({ source }: MarkdownProps) {
  return (
    <div className="prose prose-sm prose-zinc max-w-none">
      <ReactMarkdown remarkPlugins={[remarkGfm]} skipHtml>
        {source}
      </ReactMarkdown>
    </div>
  );
}
