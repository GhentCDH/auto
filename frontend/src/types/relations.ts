import type { Component } from 'vue';
import type { PaginatedResponse } from './index';

/**
 * Relation types supported by the system.
 * Each type corresponds to a different entity that can be linked to an Application or Service.
 */
export type RelationType =
  | 'infra'
  | 'service'
  | 'domain'
  | 'person'
  | 'share'
  | 'stack'
  | 'health';

/**
 * Link modal step - the 3-step flow for linking entities:
 * 1. select - Choose existing entity or request create
 * 2. create - Create new entity form
 * 3. form - Fill link metadata (notes, etc.)
 */
export type LinkStep = 'select' | 'create' | 'form';

/**
 * State for a single relation type's modals.
 * Tracks both link modal (create/link flow) and edit modal state.
 */
export interface RelationModalState<TRelation> {
  /** Whether the link modal is open */
  isLinkOpen: boolean;
  /** Whether the edit modal is open */
  isEditOpen: boolean;
  /** Entity being edited (null when not editing) */
  editing: TRelation | null;
  /** Current step in the link flow */
  linkStep: LinkStep;
  /** Selected entity during link flow */
  selectedEntity: { id: string; name: string } | null;
  /** Initial name for create form (from search input) */
  initialName: string;
}

/**
 * Entity list item returned by list APIs.
 * Used by EntitySelector to display available entities.
 */
export interface EntityListItem {
  id: string;
  name: string;
  environment?: string;
}

/**
 * Created entity response - what create APIs return.
 */
export interface CreatedEntity {
  id: string;
  name?: string;
  fqdn?: string; // For domains which use fqdn instead of name
}

/**
 * Configuration for a relation type.
 * Defines all the APIs, components, and behavior for managing a specific relation.
 *
 * @typeParam TRelation - The relation type (e.g., InfraRelation, ServiceRelation)
 * @typeParam TCreate - The create payload type (e.g., CreateInfra)
 * @typeParam TLink - The link payload type (e.g., LinkInfra)
 */
export interface RelationConfig<
  TRelation = unknown,
  TCreate = unknown,
  TLink = unknown,
> {
  /** Unique identifier for this relation type */
  type: RelationType;

  /** Display title (plural, e.g., "Infrastructure", "Services") */
  title: string;

  /** Singular title for UI (e.g., "Infra", "Service") */
  singularTitle: string;

  /** Message shown when no relations exist */
  emptyMessage: string;

  /** API to list available entities for linking */
  listApi: (params: {
    search?: string;
  }) => Promise<PaginatedResponse<EntityListItem>>;

  /** API to create a new entity */
  createApi: (data: TCreate) => Promise<CreatedEntity>;

  /**
   * API to link an entity to the parent.
   * For types without link forms (stack), data will be empty object.
   */
  linkApi: (entityId: string, data: TLink) => Promise<void>;

  /** API to unlink an entity from the parent */
  unlinkApi: (entityId: string) => Promise<void>;

  /**
   * API to update link metadata.
   * For most types, this is the same as linkApi (re-links with new data).
   * For health, this updates the healthcheck entity directly.
   */
  editApi?: (entityId: string, data: TLink) => Promise<unknown>;

  /** Form component for creating new entities */
  createFormComponent: Component;

  /** Form component for link metadata (null for direct-link types like stack (no link notes)) */
  linkFormComponent: Component | null;

  /**
   * Whether this type has a link form step.
   * - true: select → create(optional) → form → link
   * - false: select → create(optional) → link directly
   */
  hasLinkForm: boolean;

  /**
   * Whether this type is create-only (no entity selection).
   * Used for healthchecks which are always created fresh,
   * since they can't exist without a target application or service.
   */
  createOnly?: boolean;

  /**
   * Whether to show edit button for linked items.
   * Default true for most types.
   */
  canEdit?: boolean;

  /**
   * Function to get existing IDs for exclusion in entity selector.
   */
  getExcludeIds?: () => string[];

  /**
   * Display name extractor - gets display name from relation.
   * Default uses 'name' or 'fqdn' property.
   */
  getDisplayName?: (relation: TRelation) => string;

  /**
   * Custom props to pass to the create form.
   */
  getCreateFormProps?: () => Record<string, unknown>;

  /**
   * Async operation to perform before opening edit modal.
   * Used for healthchecks to fetch full entity data.
   */
  fetchForEdit?: (relation: TRelation) => Promise<unknown>;
}

/**
 * Map of all relation configs for a parent entity.
 * Uses 'any' for the generic parameters to allow assignment from specific types.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type RelationConfigMap = Partial<Record<RelationType, RelationConfig<any, any, any>>>;

/**
 * Unlink confirmation state.
 */
export interface UnlinkState {
  type: RelationType | '';
  entityId: string;
  entityName: string;
  isOpen: boolean;
}
