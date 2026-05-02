"use client";
import ReactMarkdown, { type Components } from "react-markdown";
import remarkGfm from "remark-gfm";

export interface MarkdownProps { source: string }

// Replace markdown image syntax with an inert text label so untrusted assistant
// output cannot trigger arbitrary remote image loads (privacy + tracking pixel risk).
const components: Components = {
  img: ({ alt }) => <span className="text-zinc-500 italic">[image: {alt ?? "untitled"}]</span>,
};

export function Markdown({ source }: MarkdownProps) {
  return (
    <div className="prose prose-sm prose-zinc max-w-none">
      <ReactMarkdown remarkPlugins={[remarkGfm]} skipHtml components={components}>
        {source}
      </ReactMarkdown>
    </div>
  );
}
