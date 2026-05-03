"use client";
import { useSearchParams, useRouter } from "next/navigation";
import { useCallback, useState } from "react";
import { Sidebar } from "./Sidebar";
import { ChatWindow } from "./ChatWindow";

export function ChatShell() {
  const params = useSearchParams();
  const router = useRouter();
  const activeId = params.get("c");
  const [drawerOpen, setDrawerOpen] = useState(false);

  const setActive = useCallback(
    (id: string | null) => {
      const next = new URLSearchParams(params.toString());
      if (id) next.set("c", id);
      else next.delete("c");
      router.replace(`/?${next.toString()}`);
      setDrawerOpen(false);
    },
    [params, router],
  );

  return (
    <>
      <Sidebar
        activeId={activeId}
        onSelect={setActive}
        drawerOpen={drawerOpen}
        onToggleDrawer={() => setDrawerOpen((v) => !v)}
      />
      <ChatWindow conversationId={activeId} onConversationCreated={setActive} />
    </>
  );
}
