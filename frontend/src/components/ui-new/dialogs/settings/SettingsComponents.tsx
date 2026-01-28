import { cn } from '@/lib/utils';
import { useTranslation } from 'react-i18next';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DropdownMenuTriggerButton,
} from '../../primitives/Dropdown';
import { PrimaryButton } from '../../primitives/PrimaryButton';

// SettingsCard - A card container for a settings subsection
export function SettingsCard({
  title,
  description,
  children,
  headerAction,
}: {
  title: string;
  description?: string;
  children: React.ReactNode;
  headerAction?: React.ReactNode;
}) {
  return (
    <div className="space-y-4 pb-6 border-b border-border last:border-b-0 last:pb-0">
      <div className="flex items-start justify-between">
        <div>
          <h3 className="text-base font-medium text-high">{title}</h3>
          {description && (
            <p className="text-sm text-low mt-1">{description}</p>
          )}
        </div>
        {headerAction}
      </div>
      <div className="space-y-4">{children}</div>
    </div>
  );
}

// SettingsField - A labeled field wrapper
export function SettingsField({
  label,
  description,
  error,
  children,
}: {
  label: string;
  description?: React.ReactNode;
  error?: string | null;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-2">
      {label && (
        <label className="text-sm font-medium text-normal">{label}</label>
      )}
      {children}
      {error && <p className="text-sm text-error">{error}</p>}
      {description && !error && (
        <p className="text-sm text-low">{description}</p>
      )}
    </div>
  );
}

// SettingsCheckbox - A checkbox with label and optional description
export function SettingsCheckbox({
  id,
  label,
  description,
  checked,
  onChange,
  disabled,
}: {
  id: string;
  label: string;
  description?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <div className="flex items-center gap-3">
      <input
        type="checkbox"
        id={id}
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        disabled={disabled}
        className={cn(
          'mt-0.5 h-4 w-4 rounded border-border bg-secondary text-brand focus:ring-brand focus:ring-offset-0',
          disabled && 'opacity-50 cursor-not-allowed'
        )}
      />
      <div className="space-y-0.5">
        <label
          htmlFor={id}
          className={cn(
            'text-sm font-medium text-normal cursor-pointer',
            disabled && 'opacity-50 cursor-not-allowed'
          )}
        >
          {label}
        </label>
        {description && <p className="text-sm text-low">{description}</p>}
      </div>
    </div>
  );
}

// SettingsSelect - A dropdown select component
export function SettingsSelect<T extends string>({
  value,
  options,
  onChange,
  placeholder,
  disabled,
  className,
}: {
  value: T | undefined;
  options: { value: T; label: string }[];
  onChange: (value: T) => void;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
}) {
  const selectedOption = options.find((opt) => opt.value === value);
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <DropdownMenuTriggerButton
          label={selectedOption?.label || placeholder}
          className={cn('w-full justify-between', className)}
          disabled={disabled}
        />
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-[var(--radix-dropdown-menu-trigger-width)]">
        {options.map((option) => (
          <DropdownMenuItem
            key={option.value}
            onClick={() => onChange(option.value)}
          >
            {option.label}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

// SettingsInput - A text input field
export function SettingsInput({
  value,
  onChange,
  placeholder,
  error,
  disabled,
}: {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  error?: boolean;
  disabled?: boolean;
}) {
  return (
    <input
      type="text"
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      disabled={disabled}
      className={cn(
        'w-full bg-secondary border rounded-sm px-base py-half text-sm text-high',
        'placeholder:text-low placeholder:opacity-80 focus:outline-none focus:ring-1 focus:ring-brand',
        error ? 'border-error' : 'border-border',
        disabled && 'opacity-50 cursor-not-allowed'
      )}
    />
  );
}

// SettingsTextarea - A multi-line text input
export function SettingsTextarea({
  value,
  onChange,
  placeholder,
  disabled,
  rows = 4,
  monospace = false,
}: {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  rows?: number;
  monospace?: boolean;
}) {
  return (
    <textarea
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      disabled={disabled}
      rows={rows}
      className={cn(
        'w-full bg-secondary border border-border rounded-sm px-base py-half text-sm text-high',
        'placeholder:text-low placeholder:opacity-80 focus:outline-none focus:ring-1 focus:ring-brand',
        'resize-y',
        monospace && 'font-mono',
        disabled && 'opacity-50 cursor-not-allowed'
      )}
    />
  );
}

// Predefined project colors
const PROJECT_COLORS = [
  '#E57373', // Red
  '#F06292', // Pink
  '#BA68C8', // Purple
  '#9575CD', // Deep Purple
  '#7986CB', // Indigo
  '#64B5F6', // Blue
  '#4FC3F7', // Light Blue
  '#4DD0E1', // Cyan
  '#4DB6AC', // Teal
  '#81C784', // Green
  '#AED581', // Light Green
  '#DCE775', // Lime
  '#FFF176', // Yellow
  '#FFD54F', // Amber
  '#FFB74D', // Orange
  '#FF8A65', // Deep Orange
];

// SettingsColorPicker - A color picker for project header colors
export function SettingsColorPicker({
  value,
  onChange,
  disabled,
}: {
  value: string | null;
  onChange: (value: string | null) => void;
  disabled?: boolean;
}) {
  return (
    <div className="flex flex-wrap gap-2 items-center">
      {/* Clear button */}
      <button
        type="button"
        onClick={() => onChange(null)}
        disabled={disabled}
        className={cn(
          'w-8 h-8 rounded-sm border-2 flex items-center justify-center',
          'bg-secondary hover:bg-tertiary transition-colors',
          value === null
            ? 'border-brand ring-2 ring-brand/30'
            : 'border-border',
          disabled && 'opacity-50 cursor-not-allowed'
        )}
        title="No color (default)"
      >
        <span className="text-low text-xs">∅</span>
      </button>

      {/* Color swatches */}
      {PROJECT_COLORS.map((color) => (
        <button
          key={color}
          type="button"
          onClick={() => onChange(color)}
          disabled={disabled}
          className={cn(
            'w-8 h-8 rounded-sm border-2 transition-all',
            value === color
              ? 'border-brand ring-2 ring-brand/30 scale-110'
              : 'border-border hover:border-normal hover:scale-105',
            disabled && 'opacity-50 cursor-not-allowed'
          )}
          style={{ backgroundColor: color }}
          title={color}
        />
      ))}

      {/* Custom color input */}
      <div className="flex items-center gap-1 ml-2">
        <input
          type="color"
          value={value || '#808080'}
          onChange={(e) => onChange(e.target.value)}
          disabled={disabled}
          className={cn(
            'w-8 h-8 rounded-sm border border-border cursor-pointer',
            disabled && 'opacity-50 cursor-not-allowed'
          )}
          title="Custom color"
        />
        {value && !PROJECT_COLORS.includes(value) && (
          <span className="text-xs text-low font-mono">{value}</span>
        )}
      </div>
    </div>
  );
}

// SettingsSaveBar - A sticky save bar for unsaved changes
export function SettingsSaveBar({
  show,
  saving,
  saveDisabled,
  onSave,
  onDiscard,
}: {
  show: boolean;
  saving: boolean;
  saveDisabled?: boolean;
  unsavedMessage?: string;
  onSave: () => void;
  onDiscard?: () => void;
}) {
  const { t } = useTranslation(['settings', 'common']);

  if (!show) {
    return <div />;
  }

  return (
    <div className="sticky bottom-0 z-10 bg-panel/80 backdrop-blur-sm border-t border-border/50 py-4 -mx-6 px-6 -mb-6">
      <div
        className={cn(
          'flex items-center',
          onDiscard ? 'justify-between' : 'justify-end'
        )}
      >
        {onDiscard && (
          <span className="text-sm text-low">
            {t('settings.common.unsavedChanges')}
          </span>
        )}
        <div className="flex gap-2">
          {onDiscard && (
            <PrimaryButton
              variant="tertiary"
              value={t('common:buttons.discard')}
              onClick={onDiscard}
              disabled={saving}
            />
          )}
          <PrimaryButton
            value={t('common:buttons.save')}
            onClick={onSave}
            disabled={saving || saveDisabled}
            actionIcon={saving ? 'spinner' : undefined}
          />
        </div>
      </div>
    </div>
  );
}
