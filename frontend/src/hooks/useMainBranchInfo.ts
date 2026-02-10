import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { repoApi } from '@/lib/api';
import { useProjectRepos } from '@/hooks/useProjectRepos';
import type { MainBranchInfo } from 'shared/types';

export type RepoMainBranchInfo = MainBranchInfo & { repoId: string };

export function useMainBranchInfo(projectId?: string) {
  const { data: repos } = useProjectRepos(projectId);
  const queryClient = useQueryClient();

  const firstRepo = repos?.[0];

  const query = useQuery<RepoMainBranchInfo | null>({
    queryKey: ['mainBranchInfo', projectId, firstRepo?.id],
    queryFn: async () => {
      if (!firstRepo) return null;
      const info = await repoApi.getMainBranchInfo(firstRepo.id);
      return { ...info, repoId: firstRepo.id };
    },
    enabled: !!firstRepo,
    refetchInterval: 60_000,
  });

  const pullMutation = useMutation({
    mutationFn: async () => {
      if (!firstRepo) throw new Error('No repository');
      return repoApi.pullMain(firstRepo.id);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['mainBranchInfo', projectId],
      });
    },
  });

  return {
    info: query.data ?? null,
    isLoading: query.isLoading,
    pull: pullMutation.mutate,
    isPulling: pullMutation.isPending,
    pullResult: pullMutation.data ?? null,
    pullError: pullMutation.error,
  };
}
