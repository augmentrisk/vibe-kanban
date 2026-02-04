import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card.tsx';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu.tsx';
import { Button } from '@/components/ui/button.tsx';
import { UserAvatar } from '@/components/tasks/UserAvatar';
import {
  Calendar,
  Edit,
  ExternalLink,
  MoreHorizontal,
  Trash2,
} from 'lucide-react';
import { Project, ProjectWithCreator } from 'shared/types';
import { useEffect, useRef } from 'react';
import { useNavigateWithSearch } from '@/hooks';
import { projectsApi } from '@/lib/api';
import { useTranslation } from 'react-i18next';

type Props = {
  project: ProjectWithCreator;
  isFocused: boolean;
  setError: (error: string) => void;
  onEdit: (project: Project) => void;
};

function ProjectCard({ project, isFocused, setError, onEdit }: Props) {
  const { creator } = project;
  const navigate = useNavigateWithSearch();
  const ref = useRef<HTMLDivElement>(null);
  const { t } = useTranslation('projects');

  useEffect(() => {
    if (isFocused && ref.current) {
      ref.current.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
      ref.current.focus();
    }
  }, [isFocused]);

  const handleDelete = async (id: string, name: string) => {
    if (
      !confirm(
        `Are you sure you want to delete "${name}"? This action cannot be undone.`
      )
    )
      return;

    try {
      await projectsApi.delete(id);
    } catch (error) {
      console.error('Failed to delete project:', error);
      setError('Failed to delete project');
    }
  };

  const handleEdit = (project: Project) => {
    onEdit(project);
  };

  return (
    <Card
      className={`hover:shadow-md transition-shadow cursor-pointer focus:ring-2 focus:ring-primary outline-none border relative overflow-hidden`}
      onClick={() => navigate(`/projects/${project.id}/tasks`)}
      tabIndex={isFocused ? 0 : -1}
      ref={ref}
    >
      {/* Project color indicator */}
      {project.color && (
        <div
          className="absolute top-0 left-0 right-0 h-1"
          style={{ backgroundColor: project.color }}
          aria-hidden="true"
        />
      )}
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            {/* Color swatch indicator */}
            {project.color && (
              <div
                className="w-3 h-3 rounded-full flex-shrink-0"
                style={{ backgroundColor: project.color }}
                aria-hidden="true"
              />
            )}
            <CardTitle className="text-lg">{project.name}</CardTitle>
          </div>
          <div className="flex items-center gap-2">
            <DropdownMenu>
              <DropdownMenuTrigger asChild onClick={(e) => e.stopPropagation()}>
                <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                  <MoreHorizontal className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem
                  onClick={(e) => {
                    e.stopPropagation();
                    navigate(`/projects/${project.id}`);
                  }}
                >
                  <ExternalLink className="mr-2 h-4 w-4" />
                  {t('viewProject')}
                </DropdownMenuItem>
                <DropdownMenuItem
                  onClick={(e) => {
                    e.stopPropagation();
                    handleEdit(project);
                  }}
                >
                  <Edit className="mr-2 h-4 w-4" />
                  {t('common:buttons.edit')}
                </DropdownMenuItem>
                <DropdownMenuItem
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDelete(project.id, project.name);
                  }}
                  className="text-destructive"
                >
                  <Trash2 className="mr-2 h-4 w-4" />
                  {t('common:buttons.delete')}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
        <CardDescription className="flex items-center gap-2">
          <span className="flex items-center">
            <Calendar className="mr-1 h-3 w-3" />
            {t('createdDate', {
              date: new Date(project.created_at).toLocaleDateString(),
            })}
          </span>
          {creator && (
            <span className="flex items-center gap-1">
              <UserAvatar
                username={creator.username}
                imageUrl={creator.avatar_url}
                className="h-4 w-4"
              />
              <span className="text-xs text-muted-foreground">
                {creator.username}
              </span>
            </span>
          )}
        </CardDescription>
      </CardHeader>
    </Card>
  );
}

export default ProjectCard;
