import { createFetch, createSchema } from '@better-fetch/fetch';
import { z } from 'zod';
import { env } from '../env';

const timeRangeQuery = z.object({
  start: z.number().optional(),
  end: z.number().optional(),
});

const totalOverviewSchema = z.object({
  total_runs: z.number(),
  total_awake_time_ms: z.number(),
  total_sleep_time_ms: z.number(),
  total_start_failures: z.number(),
});

const appOverviewSchema = z.object({
  host: z.string(),
  total_runs: z.number(),
  total_awake_time_ms: z.number(),
  total_sleep_time_ms: z.number(),
  total_start_failures: z.number(),
});

const appRunSchema = z.object({
  start_time_ms: z.number(),
  end_time_ms: z.number(),
  total_awake_time_ms: z.number(),
});

export const schema = createSchema(
  {
    '/api/version': {
      output: z.object({ version: z.string() }),
    },
    '/api/total-overview': {
      query: timeRangeQuery,
      output: totalOverviewSchema,
    },
    '/api/apps-overview': {
      query: timeRangeQuery,
      output: z.array(appOverviewSchema),
    },
    '/api/app-overview/:host': {
      params: z.object({
        host: z.string(),
      }),
      query: timeRangeQuery,
      output: appOverviewSchema,
    },
    '/api/app-runs/:host': {
      params: z.object({
        host: z.string(),
      }),
      query: timeRangeQuery,
      output: z.array(appRunSchema),
    },
  },
  { strict: true },
);

export const $fetch = createFetch({
  baseURL: env.VITE_API_URL ?? '',
  schema,
  throw: true,
});

export type TimeRange = z.infer<typeof timeRangeQuery>;
export type TotalOverview = z.infer<typeof totalOverviewSchema>;
export type AppOverview = z.infer<typeof appOverviewSchema>;
export type AppRun = z.infer<typeof appRunSchema>;
