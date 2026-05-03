"use client";
import clsx from "clsx";
import { Plus, Menu, Loader } from "lucide-react";
import { useConversations } from "@/hooks/useConversations";
import { useCreateConversation } from "@/hooks/useCreateConversation";
import { ConversationItem } from "./ConversationItem";

export interface SidebarProps {
  activeId: string | null;
  onSelect: (id: string | null) => void;
  drawerOpen: boolean;
  onToggleDrawer: () => void;
}

export function Sidebar(props: SidebarProps) {
  const { activeId, onSelect, drawerOpen, onToggleDrawer } = props;
  const list = useConversations();
  const create = useCreateConversation();

  const onNewChat = async () => {
    const c = await create.mutateAsync({ title: "New chat" });
    onSelect(c.id);
  };

  return (
    <>
      <button
        type="button"
        aria-label="Toggle sidebar"
        className="md:hidden absolute left-3 top-3 z-20 rounded p-2 bg-zinc-800"
        onClick={onToggleDrawer}
      >
        <Menu size={18} />
      </button>
      <aside
        data-testid="sidebar"
        className={clsx(
          "w-64 shrink-0 border-r border-zinc-800 bg-zinc-900 p-3 flex flex-col gap-3",
          "md:block",
          drawerOpen ? "absolute inset-y-0 left-0 z-10 block" : "hidden md:flex",
        )}
      >
        <button
          type="button"
          onClick={onNewChat}
          disabled={create.isPending}
          className="flex items-center gap-2 rounded bg-zinc-800 px-3 py-2 text-sm hover:bg-zinc-700 disabled:opacity-50"
        >
          {create.isPending ? <Loader size={16} className="animate-spin" /> : <Plus size={16} />}
          New chat
        </button>
        <ul className="flex-1 overflow-y-auto space-y-1">
          {list.data?.map((c) => (
            <ConversationItem
              key={c.id}
              conversation={c}
              active={c.id === activeId}
              onSelect={onSelect}
            />
          ))}
          {list.isLoading && <li className="text-xs text-zinc-500">Loading…</li>}
          {list.isError && (
            <li className="text-xs text-red-400">Failed to load conversations.</li>
          )}
        </ul>
      </aside>
    </>
  );
}
