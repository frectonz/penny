import { format } from 'date-fns';
import { CalendarIcon } from 'lucide-react';
import { useState } from 'react';
import { Button } from './ui/button';
import { Calendar } from './ui/calendar';
import { Popover, PopoverContent, PopoverTrigger } from './ui/popover';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select';

export type TimeRangeValue = {
  start?: number;
  end?: number;
};

type PresetRange =
  | 'last-hour'
  | 'last-24h'
  | 'last-7d'
  | 'last-30d'
  | 'all-time'
  | 'custom';

const PRESETS: { value: PresetRange; label: string }[] = [
  { value: 'last-hour', label: 'Last hour' },
  { value: 'last-24h', label: 'Last 24 hours' },
  { value: 'last-7d', label: 'Last 7 days' },
  { value: 'last-30d', label: 'Last 30 days' },
  { value: 'all-time', label: 'All time' },
  { value: 'custom', label: 'Custom' },
];

function getPresetRange(preset: PresetRange): TimeRangeValue {
  const now = Date.now();

  switch (preset) {
    case 'last-hour':
      return { start: now - 60 * 60 * 1000, end: now };
    case 'last-24h':
      return { start: now - 24 * 60 * 60 * 1000, end: now };
    case 'last-7d':
      return { start: now - 7 * 24 * 60 * 60 * 1000, end: now };
    case 'last-30d':
      return { start: now - 30 * 24 * 60 * 60 * 1000, end: now };
    case 'all-time':
      return { start: undefined, end: undefined };
    case 'custom':
      return { start: undefined, end: undefined };
  }
}

function detectPreset(value: TimeRangeValue): PresetRange {
  if (value.start === undefined && value.end === undefined) {
    return 'all-time';
  }

  if (value.start === undefined || value.end === undefined) {
    return 'custom';
  }

  const now = Date.now();
  const diff = value.end - value.start;
  const endDiff = Math.abs(now - value.end);

  // Allow 1 minute tolerance for "now"
  const isRecent = endDiff < 60 * 1000;

  if (!isRecent) {
    return 'custom';
  }

  const hour = 60 * 60 * 1000;
  const day = 24 * hour;

  // Allow 1 minute tolerance for preset detection
  const tolerance = 60 * 1000;

  if (Math.abs(diff - hour) < tolerance) return 'last-hour';
  if (Math.abs(diff - day) < tolerance) return 'last-24h';
  if (Math.abs(diff - 7 * day) < tolerance) return 'last-7d';
  if (Math.abs(diff - 30 * day) < tolerance) return 'last-30d';

  return 'custom';
}

interface TimeRangeSelectorProps {
  value: TimeRangeValue;
  onChange: (value: TimeRangeValue) => void;
}

export function TimeRangeSelector({ value, onChange }: TimeRangeSelectorProps) {
  const [isCustom, setIsCustom] = useState(
    () => detectPreset(value) === 'custom',
  );
  const currentPreset = isCustom ? 'custom' : detectPreset(value);

  const handlePresetChange = (preset: PresetRange) => {
    if (preset === 'custom') {
      setIsCustom(true);
      return;
    }

    setIsCustom(false);
    onChange(getPresetRange(preset));
  };

  const handleStartDateChange = (date: Date | undefined) => {
    onChange({
      ...value,
      start: date ? date.getTime() : undefined,
    });
  };

  const handleEndDateChange = (date: Date | undefined) => {
    onChange({
      ...value,
      end: date ? date.getTime() : undefined,
    });
  };

  const startDate = value.start ? new Date(value.start) : undefined;
  const endDate = value.end ? new Date(value.end) : undefined;

  return (
    <div className="flex items-center gap-2">
      <Select value={currentPreset} onValueChange={handlePresetChange}>
        <SelectTrigger className="w-[140px]" size="sm">
          <SelectValue placeholder="Select range" />
        </SelectTrigger>
        <SelectContent>
          {PRESETS.map((preset) => (
            <SelectItem key={preset.value} value={preset.value}>
              {preset.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      {isCustom && (
        <>
          <Popover>
            <PopoverTrigger asChild>
              <Button
                variant="outline"
                size="sm"
                className="justify-start text-left font-normal"
              >
                <CalendarIcon className="mr-1 h-3 w-3" />
                {startDate ? format(startDate, 'MMM d, yyyy') : 'Start date'}
              </Button>
            </PopoverTrigger>
            <PopoverContent className="w-auto p-0" align="end">
              <Calendar
                mode="single"
                selected={startDate}
                onSelect={handleStartDateChange}
                disabled={(date) => (endDate ? date > endDate : false)}
              />
            </PopoverContent>
          </Popover>

          <span className="text-muted-foreground text-sm">to</span>

          <Popover>
            <PopoverTrigger asChild>
              <Button
                variant="outline"
                size="sm"
                className="justify-start text-left font-normal"
              >
                <CalendarIcon className="mr-1 h-3 w-3" />
                {endDate ? format(endDate, 'MMM d, yyyy') : 'End date'}
              </Button>
            </PopoverTrigger>
            <PopoverContent className="w-auto p-0" align="end">
              <Calendar
                mode="single"
                selected={endDate}
                onSelect={handleEndDateChange}
                disabled={(date) => (startDate ? date < startDate : false)}
              />
            </PopoverContent>
          </Popover>
        </>
      )}
    </div>
  );
}
