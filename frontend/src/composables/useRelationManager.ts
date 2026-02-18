import { reactive, ref, computed } from 'vue';
import type {
  RelationType,
  RelationModalState,
  RelationConfigMap,
  UnlinkState,
  LinkStep,
} from '@/types/relations';

/**
 * Options for the relation manager composable.
 */
export interface RelationManagerOptions {
  /** Callback when a relation operation succeeds */
  onSuccess: () => void;
  /** Callback when a relation operation fails */
  onError: (message: string) => void;
}

/**
 * Creates initial modal state for a relation type.
 */
function createInitialModalState<T>(): RelationModalState<T> {
  return {
    isLinkOpen: false,
    isEditOpen: false,
    editing: null,
    linkStep: 'select',
    selectedEntity: null,
    initialName: '',
  };
}

/**
 * Composable for managing relation modals and operations.
 *
 * Replaces the repetitive pattern of:
 * - 7 showLink*Modal refs
 * - 7 showEdit*Modal refs
 * - 7 editing* refs
 * - openLinkModal/closeLinkModal switch statements
 * - individual create/link/edit handlers for each type
 *
 * @example
 * ```ts
 * const { modalStates, openLinkModal, handlers } = useRelationManager(configs, {
 *   onSuccess: () => loadData(),
 *   onError: (msg) => error.value = msg,
 * });
 *
 * // In template:
 * // <button @click="openLinkModal('infra')">Add</button>
 * // <RelationLinkModal :open="modalStates.infra.isLinkOpen" ... />
 * ```
 */
export function useRelationManager(
  configs: RelationConfigMap,
  options: RelationManagerOptions
) {
  const { onSuccess, onError } = options;

  // Create reactive modal states for each configured relation type
  const modalStates = reactive<
    Record<RelationType, RelationModalState<unknown>>
  >({
    infra: createInitialModalState(),
    service: createInitialModalState(),
    domain: createInitialModalState(),
    person: createInitialModalState(),
    share: createInitialModalState(),
    stack: createInitialModalState(),
    health: createInitialModalState(),
  });

  // Unlink confirmation state
  const unlinkState = reactive<UnlinkState>({
    type: '',
    entityId: '',
    entityName: '',
    isOpen: false,
  });

  // Track loading states for async operations
  const isLoading = ref(false);

  /**
   * Opens the link modal for a relation type.
   * Resets modal state to initial 'select' step.
   */
  function openLinkModal(type: RelationType) {
    const state = modalStates[type];
    state.linkStep = 'select';
    state.selectedEntity = null;
    state.initialName = '';
    state.isLinkOpen = true;
  }

  /**
   * Closes the link modal for a relation type.
   */
  function closeLinkModal(type: RelationType) {
    modalStates[type].isLinkOpen = false;
  }

  /**
   * Opens the edit modal for a relation.
   * For some types (healthcheck), fetches additional data first.
   */
  async function openEditModal(type: RelationType, entity: unknown) {
    const config = configs[type];
    if (!config) return;

    const state = modalStates[type];

    // Some types need to fetch full entity data before editing
    if (config.fetchForEdit) {
      try {
        isLoading.value = true;
        const fullEntity = await config.fetchForEdit(entity);
        state.editing = fullEntity;
      } catch (e) {
        onError(e instanceof Error ? e.message : `Failed to load ${type}`);
        return;
      } finally {
        isLoading.value = false;
      }
    } else {
      state.editing = entity;
    }

    state.isEditOpen = true;
  }

  /**
   * Closes the edit modal for a relation type.
   */
  function closeEditModal(type: RelationType) {
    const state = modalStates[type];
    state.isEditOpen = false;
    state.editing = null;
  }

  /**
   * Sets the link step for a relation type.
   */
  function setLinkStep(type: RelationType, step: LinkStep) {
    modalStates[type].linkStep = step;
  }

  /**
   * Handles entity selection in the link flow.
   * For types with link forms, advances to 'form' step.
   * For direct-link types (stack), links immediately.
   */
  async function handleEntitySelect(
    type: RelationType,
    entity: { id: string; name: string }
  ) {
    const config = configs[type];
    if (!config) return;

    const state = modalStates[type];

    if (config.hasLinkForm) {
      // Advance to link form
      state.selectedEntity = entity;
      state.linkStep = 'form';
    } else {
      // Direct link (no form step)
      try {
        isLoading.value = true;
        await config.linkApi(entity.id, {});
        state.isLinkOpen = false;
        onSuccess();
      } catch (e) {
        onError(
          e instanceof Error
            ? e.message
            : `Failed to link ${config.singularTitle}`
        );
      } finally {
        isLoading.value = false;
      }
    }
  }

  /**
   * Handles create request from entity selector.
   * Stores initial name and advances to 'create' step.
   */
  function handleCreateRequest(type: RelationType, searchTerm: string) {
    const state = modalStates[type];
    state.initialName = searchTerm;
    state.linkStep = 'create';
  }

  /**
   * Creates a new entity and handles the post-create flow.
   * - For types with link forms: advances to 'form' step
   * - For direct-link types: links immediately
   * - For create-only types: closes modal
   */
  async function handleCreate<TCreate>(type: RelationType, data: TCreate) {
    const config = configs[type];
    if (!config) return;

    const state = modalStates[type];

    try {
      isLoading.value = true;
      const created = await config.createApi(data);

      // Determine display name from response
      const name = created.name || created.fqdn || '';

      if (config.createOnly) {
        // Create-only types (like health) - just close modal
        state.isLinkOpen = false;
        onSuccess();
      } else if (config.hasLinkForm) {
        // Advance to link form
        state.selectedEntity = { id: created.id, name };
        state.linkStep = 'form';
      } else {
        // Direct link after create (like stack)
        await config.linkApi(created.id, {});
        state.isLinkOpen = false;
        onSuccess();
      }
    } catch (e) {
      onError(
        e instanceof Error
          ? e.message
          : `Failed to create ${config.singularTitle}`
      );
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Submits link form data to link the selected entity.
   */
  async function handleLink<TLink>(type: RelationType, data: TLink) {
    const config = configs[type];
    if (!config) return;

    const state = modalStates[type];
    if (!state.selectedEntity) return;

    try {
      isLoading.value = true;
      await config.linkApi(state.selectedEntity.id, data);
      state.isLinkOpen = false;
      onSuccess();
    } catch (e) {
      onError(
        e instanceof Error
          ? e.message
          : `Failed to link ${config.singularTitle}`
      );
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Submits edit form data to update link metadata.
   */
  async function handleEdit<TLink>(type: RelationType, data: TLink) {
    const config = configs[type];
    if (!config) return;

    const state = modalStates[type];
    const editing = state.editing as { id: string } | null;
    if (!editing) return;

    try {
      isLoading.value = true;
      const editApi = config.editApi || config.linkApi;
      await editApi(editing.id, data);
      state.isEditOpen = false;
      state.editing = null;
      onSuccess();
    } catch (e) {
      onError(
        e instanceof Error
          ? e.message
          : `Failed to update ${config.singularTitle} link`
      );
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Opens unlink confirmation dialog.
   */
  function confirmUnlink(
    type: RelationType,
    entityId: string,
    entityName: string
  ) {
    unlinkState.type = type;
    unlinkState.entityId = entityId;
    unlinkState.entityName = entityName;
    unlinkState.isOpen = true;
  }

  /**
   * Performs the unlink operation after confirmation.
   */
  async function handleUnlink() {
    if (!unlinkState.type) return;

    const config = configs[unlinkState.type as RelationType];
    if (!config) return;

    try {
      isLoading.value = true;
      await config.unlinkApi(unlinkState.entityId);
      unlinkState.isOpen = false;
      onSuccess();
    } catch (e) {
      onError(e instanceof Error ? e.message : 'Failed to unlink');
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Cancels the unlink confirmation.
   */
  function cancelUnlink() {
    unlinkState.isOpen = false;
  }

  /**
   * Computed unlink message for confirmation dialog.
   */
  const unlinkMessage = computed(() => {
    if (unlinkState.type === 'health') {
      return `Are you sure you want to remove healthcheck '${unlinkState.entityName}'?`;
    }
    return `Are you sure you want to unlink '${unlinkState.entityName}' from this application?`;
  });

  return {
    // State
    modalStates,
    unlinkState,
    unlinkMessage,
    isLoading,

    // Link modal operations
    openLinkModal,
    closeLinkModal,
    setLinkStep,
    handleEntitySelect,
    handleCreateRequest,
    handleCreate,
    handleLink,

    // Edit modal operations
    openEditModal,
    closeEditModal,
    handleEdit,

    // Unlink operations
    confirmUnlink,
    handleUnlink,
    cancelUnlink,
  };
}
