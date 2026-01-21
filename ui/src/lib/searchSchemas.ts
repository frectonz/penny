import { z } from 'zod';

export const timeRangeSearchSchema = z.object({
  start: z.number().optional(),
  end: z.number().optional(),
});

export type TimeRangeSearch = z.infer<typeof timeRangeSearchSchema>;
