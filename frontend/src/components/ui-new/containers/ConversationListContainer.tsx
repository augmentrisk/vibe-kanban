import { useEffect, useRef, useState } from 'react';
import { Virtuoso, VirtuosoHandle } from 'react-virtuoso';

import { cn } from '@/lib/utils';
import NewDisplayConversationEntry from './NewDisplayConversationEntry';
import { ApprovalFormProvider } from '@/contexts/ApprovalFormContext';
import { useEntries } from '@/contexts/EntriesContext';
import {
  AddEntryType,
  PatchTypeWithKey,
  useConversationHistory,
} from '@/components/ui-new/hooks/useConversationHistory';
import type { WorkspaceWithSession } from '@/types/attempt';

interface ConversationListProps {
  attempt: WorkspaceWithSession;
}

export function ConversationList({ attempt }: ConversationListProps) {
  const [entries, setEntriesState] = useState<PatchTypeWithKey[]>([]);
  const [loading, setLoading] = useState(true);
  const { setEntries, reset } = useEntries();
  const virtuosoRef = useRef<VirtuosoHandle>(null);
  const [atBottom, setAtBottom] = useState(true);
  const didInitScroll = useRef(false);
  const pendingUpdateRef = useRef<{
    entries: PatchTypeWithKey[];
    addType: AddEntryType;
    loading: boolean;
  } | null>(null);
  const debounceTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    setLoading(true);
    setEntriesState([]);
    reset();
    didInitScroll.current = false;
  }, [attempt.id, reset]);

  useEffect(() => {
    return () => {
      if (debounceTimeoutRef.current) {
        clearTimeout(debounceTimeoutRef.current);
      }
    };
  }, []);

  const onEntriesUpdated = (
    newEntries: PatchTypeWithKey[],
    addType: AddEntryType,
    newLoading: boolean
  ) => {
    pendingUpdateRef.current = {
      entries: newEntries,
      addType,
      loading: newLoading,
    };

    if (debounceTimeoutRef.current) {
      clearTimeout(debounceTimeoutRef.current);
    }

    debounceTimeoutRef.current = setTimeout(() => {
      const pending = pendingUpdateRef.current;
      if (!pending) return;

      setEntriesState(pending.entries);
      setEntries(pending.entries);

      if (loading) {
        setLoading(pending.loading);
      }

      // On initial load, jump to the bottom
      if (
        pending.addType === 'initial' &&
        !didInitScroll.current &&
        pending.entries.length > 0
      ) {
        didInitScroll.current = true;
        requestAnimationFrame(() => {
          virtuosoRef.current?.scrollToIndex({
            index: pending.entries.length - 1,
            align: 'end',
          });
        });
      }

      // On plan updates, scroll so the last item's top is visible
      if (pending.addType === 'plan' && pending.entries.length > 0) {
        requestAnimationFrame(() => {
          virtuosoRef.current?.scrollToIndex({
            index: pending.entries.length - 1,
            align: 'start',
          });
        });
      }
    }, 100);
  };

  useConversationHistory({ attempt, onEntriesUpdated });

  // Determine if content is ready to show (has data or finished loading)
  const hasContent = !loading || entries.length > 0;

  const itemContent = (_index: number, data: PatchTypeWithKey) => {
    if (data.type === 'STDOUT') {
      return <p>{data.content}</p>;
    }
    if (data.type === 'STDERR') {
      return <p>{data.content}</p>;
    }
    if (data.type === 'NORMALIZED_ENTRY') {
      return (
        <NewDisplayConversationEntry
          expansionKey={data.patchKey}
          entry={data.content}
          executionProcessId={data.executionProcessId}
          taskAttempt={attempt}
        />
      );
    }

    return null;
  };

  return (
    <ApprovalFormProvider>
      <div
        className={cn(
          'h-full transition-opacity duration-300',
          hasContent ? 'opacity-100' : 'opacity-0'
        )}
      >
        <Virtuoso<PatchTypeWithKey>
          ref={virtuosoRef}
          className="h-full scrollbar-none"
          data={entries}
          itemContent={itemContent}
          computeItemKey={(_, data) => `conv-${data.patchKey}`}
          atBottomStateChange={setAtBottom}
          followOutput={atBottom ? 'smooth' : false}
          increaseViewportBy={{ top: 0, bottom: 600 }}
          components={{
            Header: () => <div className="h-2" />,
            Footer: () => <div className="h-2" />,
          }}
        />
      </div>
    </ApprovalFormProvider>
  );
}

export default ConversationList;
