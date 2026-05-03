import { Suspense } from "react";
import { ChatShell } from "@/components/ChatShell";

export default function Page() {
  return (
    <main className="flex h-full w-full flex-row overflow-hidden">
      <Suspense fallback={<div className="flex-1" />}>
        <ChatShell />
      </Suspense>
    </main>
  );
}
