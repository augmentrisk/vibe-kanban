import { Diff } from 'shared/types';
import type { ConversationWithMessages } from 'shared/types';
import { DiffModeEnum, DiffView, SplitSide } from '@git-diff-view/react';
import { generateDiffFile, type DiffFile } from '@git-diff-view/file';
import { useCallback, useMemo } from 'react';
import { useUserSystem } from '@/components/ConfigProvider';
import { getHighLightLanguageFromPath } from '@/utils/extToLanguage';
import { getActualTheme } from '@/utils/theme';
import { stripLineEnding } from '@/utils/string';
import { Button } from '@/components/ui/button';
import {
  ChevronRight,
  ChevronUp,
  Trash2,
  ArrowLeftRight,
  FilePlus2,
  PencilLine,
  Copy,
  Key,
  MessageSquare,
} from 'lucide-react';
import '@/styles/diff-style-overrides.css';
import type { Workspace } from 'shared/types';
import {
  useReview,
  diffSideToSplitSide,
  type ReviewDraft,
  type ReviewComment,
} from '@/contexts/ReviewProvider';
import { ConversationWidgetLine } from '@/components/diff/ConversationWidgetLine';
import { ConversationRenderer } from '@/components/diff/ConversationRenderer';
import { ReviewCommentRenderer } from '@/components/diff/ReviewCommentRenderer';
import {
  useDiffViewMode,
  useIgnoreWhitespaceDiff,
  useWrapTextDiff,
} from '@/stores/useDiffViewStore';
import { useProject } from '@/contexts/ProjectContext';

type ExtendLineData =
  | { type: 'conversation'; conversation: ConversationWithMessages }
  | { type: 'review'; comment: ReviewComment };

type Props = {
  diff: Diff;
  expanded: boolean;
  onToggle: () => void;
};

function labelAndIcon(diff: Diff) {
  const c = diff.change;
  if (c === 'deleted') return { label: 'Deleted', Icon: Trash2 };
  if (c === 'renamed') return { label: 'Renamed', Icon: ArrowLeftRight };
  if (c === 'added')
    return { label: undefined as string | undefined, Icon: FilePlus2 };
  if (c === 'copied') return { label: 'Copied', Icon: Copy };
  if (c === 'permissionChange')
    return { label: 'Permission Changed', Icon: Key };
  return { label: undefined as string | undefined, Icon: PencilLine };
}

function readPlainLine(
  diffFile: DiffFile | null,
  lineNumber: number,
  side: SplitSide
) {
  if (!diffFile) return undefined;
  try {
    const rawLine =
      side === SplitSide.old
        ? diffFile.getOldPlainLine(lineNumber)
        : diffFile.getNewPlainLine(lineNumber);
    if (rawLine?.value === undefined) return undefined;
    return stripLineEnding(rawLine.value);
  } catch (error) {
    console.error('Failed to read line content for review comment', error);
    return undefined;
  }
}

export default function DiffCard({
  diff,
  expanded,
  onToggle,
}: Props) {
  const { config } = useUserSystem();
  const theme = getActualTheme(config?.theme);
  const { comments, drafts, setDraft, getConversationsForFile } = useReview();
  const globalMode = useDiffViewMode();
  const ignoreWhitespace = useIgnoreWhitespaceDiff();
  const wrapText = useWrapTextDiff();
  const { projectId } = useProject();

  const oldName = diff.oldPath || undefined;
  const newName = diff.newPath || oldName || 'unknown';
  const oldLang =
    getHighLightLanguageFromPath(oldName || newName || '') || 'plaintext';
  const newLang =
    getHighLightLanguageFromPath(newName || oldName || '') || 'plaintext';
  const { label, Icon } = labelAndIcon(diff);
  const isOmitted = !!diff.contentOmitted;

  // Build a diff from raw contents so the viewer can expand beyond hunks
  const oldContentSafe = diff.oldContent || '';
  const newContentSafe = diff.newContent || '';
  const isContentEqual = oldContentSafe === newContentSafe;

  const diffOptions = useMemo(
    () => (ignoreWhitespace ? { ignoreWhitespace: true as const } : undefined),
    [ignoreWhitespace]
  );

  const diffFile = useMemo(() => {
    if (isContentEqual || isOmitted) return null;
    try {
      const oldFileName = oldName || newName || 'unknown';
      const newFileName = newName || oldName || 'unknown';
      const file = generateDiffFile(
        oldFileName,
        oldContentSafe,
        newFileName,
        newContentSafe,
        oldLang,
        newLang,
        diffOptions
      );
      file.initRaw();
      return file;
    } catch (e) {
      console.error('Failed to build diff for view', e);
      return null;
    }
  }, [
    isContentEqual,
    isOmitted,
    oldName,
    newName,
    oldLang,
    newLang,
    oldContentSafe,
    newContentSafe,
    diffOptions,
  ]);

  const add = isOmitted
    ? (diff.additions ?? 0)
    : (diffFile?.additionLength ?? 0);
  const del = isOmitted
    ? (diff.deletions ?? 0)
    : (diffFile?.deletionLength ?? 0);

  // Review functionality
  const filePath = newName || oldName || 'unknown';
  const commentsForFile = useMemo(
    () => comments.filter((c) => c.filePath === filePath),
    [comments, filePath]
  );
  const conversationsForFile = getConversationsForFile(filePath);

  const unresolvedConversationsCount = conversationsForFile.filter(
    (c) => !c.is_resolved
  ).length;

  // Transform comments + conversations to git-diff-view extendData format
  const extendData = useMemo(() => {
    const oldFileData: Record<string, { data: ExtendLineData }> = {};
    const newFileData: Record<string, { data: ExtendLineData }> = {};

    // Conversations first (highest priority)
    conversationsForFile.forEach((conversation) => {
      const lineKey = String(conversation.line_number);
      const entry: ExtendLineData = { type: 'conversation', conversation };
      const side = diffSideToSplitSide(conversation.side);
      if (side === SplitSide.old) {
        oldFileData[lineKey] = { data: entry };
      } else {
        newFileData[lineKey] = { data: entry };
      }
    });

    // Legacy user comments (only if no conversation on that line)
    commentsForFile.forEach((comment) => {
      const lineKey = String(comment.lineNumber);
      const entry: ExtendLineData = { type: 'review', comment };
      if (comment.side === SplitSide.old) {
        if (!oldFileData[lineKey]) {
          oldFileData[lineKey] = { data: entry };
        }
      } else {
        if (!newFileData[lineKey]) {
          newFileData[lineKey] = { data: entry };
        }
      }
    });

    return {
      oldFile: oldFileData,
      newFile: newFileData,
    };
  }, [conversationsForFile, commentsForFile]);

  const handleAddWidgetClick = (lineNumber: number, side: SplitSide) => {
    const widgetKey = `${filePath}-${side}-${lineNumber}`;
    const codeLine = readPlainLine(diffFile, lineNumber, side);
    const draft: ReviewDraft = {
      filePath,
      side,
      lineNumber,
      text: '',
      ...(codeLine !== undefined ? { codeLine } : {}),
    };
    setDraft(widgetKey, draft);
  };

  const renderWidgetLine = useCallback(
    (props: {
      side: SplitSide;
      lineNumber: number;
      onClose: () => void;
    }) => {
      const widgetKey = `${filePath}-${props.side}-${props.lineNumber}`;
      const draft = drafts[widgetKey];
      if (!draft) return null;

      return (
        <ConversationWidgetLine
          draft={draft}
          widgetKey={widgetKey}
          onSave={props.onClose}
          onCancel={props.onClose}
          projectId={projectId}
        />
      );
    },
    [filePath, drafts, projectId]
  );

  const renderExtendLine = useCallback(
    (lineData: { data: ExtendLineData }) => {
      if (!lineData.data) return null;

      if (lineData.data.type === 'conversation') {
        return (
          <ConversationRenderer
            conversation={lineData.data.conversation}
            projectId={projectId}
          />
        );
      }
      return (
        <ReviewCommentRenderer
          comment={lineData.data.comment}
          projectId={projectId}
        />
      );
    },
    [projectId]
  );

  // Title row
  const title = (
    <p
      className="text-sm font-mono overflow-x-auto flex-1"
      style={{ color: 'hsl(var(--muted-foreground) / 0.7)' }}
    >
      <Icon className="h-3 w-3 inline mr-2" aria-hidden />
      {label && <span className="mr-2">{label}</span>}
      {diff.change === 'renamed' && oldName ? (
        <span className="inline-flex items-center gap-2">
          <span>{oldName}</span>
          <span aria-hidden>→</span>
          <span>{newName}</span>
        </span>
      ) : (
        <span>{newName}</span>
      )}
      <span className="ml-3" style={{ color: 'hsl(var(--console-success))' }}>
        +{add}
      </span>
      <span className="ml-2" style={{ color: 'hsl(var(--console-error))' }}>
        -{del}
      </span>
      {(commentsForFile.length + unresolvedConversationsCount) > 0 && (
        <span className="ml-3 inline-flex items-center gap-1 px-2 py-0.5 text-xs bg-primary/10 text-primary rounded">
          <MessageSquare className="h-3 w-3" />
          {commentsForFile.length + unresolvedConversationsCount}
        </span>
      )}
    </p>
  );


  const expandable = true;

  return (
    <div className="my-4 border">
      <div className="sticky top-0 z-[5] flex items-center px-4 py-2 bg-background border-b">
        {expandable && (
          <Button
            variant="ghost"
            size="sm"
            onClick={onToggle}
            className="h-6 w-6 p-0 mr-2"
            title={expanded ? 'Collapse' : 'Expand'}
            aria-expanded={expanded}
          >
            {expanded ? (
              <ChevronUp className="h-3 w-3" />
            ) : (
              <ChevronRight className="h-3 w-3" />
            )}
          </Button>
        )}
        {title}
      </div>

      {expanded && diffFile && (
        <div>
          <DiffView
            diffFile={diffFile}
            diffViewWrap={wrapText}
            diffViewTheme={theme}
            diffViewHighlight
            diffViewMode={
              globalMode === 'split' ? DiffModeEnum.Split : DiffModeEnum.Unified
            }
            diffViewFontSize={12}
            diffViewAddWidget
            onAddWidgetClick={handleAddWidgetClick}
            renderWidgetLine={renderWidgetLine}
            extendData={extendData}
            renderExtendLine={renderExtendLine}
          />
        </div>
      )}
      {expanded && !diffFile && (
        <div
          className="px-4 pb-4 text-xs font-mono"
          style={{ color: 'hsl(var(--muted-foreground) / 0.9)' }}
        >
          {isOmitted
            ? 'Content omitted due to file size. Open in editor to view.'
            : isContentEqual
              ? diff.change === 'renamed'
                ? 'File renamed with no content changes.'
                : diff.change === 'permissionChange'
                  ? 'File permission changed.'
                  : 'No content changes to display.'
              : 'Failed to render diff for this file.'}
        </div>
      )}
    </div>
  );
}
