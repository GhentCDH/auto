/**
 * Common Zod schemas shared across tools
 */

import { z } from "zod";
import { ResponseFormat, Environment, Status, InfrastructureType, ShareType, NoteType, ContributionType, HttpMethod, Protocol } from "../constants.js";

export const PaginationSchema = z.object({
  page: z.number()
    .int()
    .min(1)
    .default(1)
    .describe("Page number (1-indexed)"),
  per_page: z.number()
    .int()
    .min(1)
    .max(100)
    .default(50)
    .describe("Number of items per page (max 100)")
}).strict();

export const ResponseFormatSchema = z.object({
  response_format: z.nativeEnum(ResponseFormat)
    .default(ResponseFormat.MARKDOWN)
    .describe("Output format: 'markdown' for human-readable or 'json' for machine-readable")
}).strict();

export const SearchSchema = z.object({
  search: z.string()
    .min(1)
    .optional()
    .describe("Search term to filter by name/description")
}).strict();

export const EnvironmentFilterSchema = z.object({
  environment: z.nativeEnum(Environment)
    .optional()
    .describe("Filter by environment (prd, dev, stg, tst)")
}).strict();

export const StatusFilterSchema = z.object({
  status: z.nativeEnum(Status)
    .optional()
    .describe("Filter by status (active, inactive, deprecated)")
}).strict();

export const IdSchema = z.object({
  id: z.string()
    .uuid()
    .describe("Resource ID (UUID format)")
}).strict();
