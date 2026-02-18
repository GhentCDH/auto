import type { RelationConfig, RelationConfigMap } from '@/types/relations';
import type {
  ApplicationWithRelations,
  ServiceWithRelations,
  CreateInfra,
  CreateService,
  CreateDomain,
  CreatePerson,
  CreateNetworkShare,
  CreateStack,
  CreateHealthcheck,
  LinkInfra,
  LinkService,
  LinkDomain,
  LinkPerson,
  LinkNetworkShare,
  UpdateHealthcheck,
  InfraRelation,
  ServiceRelation,
  DomainRelation,
  PersonRelation,
  NetworkShareRelation,
  StackRelation,
  HealthcheckRelation,
  Healthcheck,
} from '@/types';
import {
  applicationsApi,
  servicesApi,
  infraApi,
  domainsApi,
  peopleApi,
  sharesApi,
  stacksApi,
  healthchecksApi,
} from '@/api';
import { markRaw, type Component } from 'vue';

// Form components - imported dynamically to avoid circular dependencies
import InfraForm from '@/components/forms/InfraForm.vue';
import ServiceForm from '@/components/forms/ServiceForm.vue';
import DomainForm from '@/components/forms/DomainForm.vue';
import PersonForm from '@/components/forms/PersonForm.vue';
import ShareForm from '@/components/forms/ShareForm.vue';
import StackForm from '@/components/forms/StackForm.vue';
import HealthcheckForm from '@/components/forms/HealthcheckForm.vue';
import LinkInfraForm from '@/components/forms/LinkInfraForm.vue';
import LinkServiceForm from '@/components/forms/LinkServiceForm.vue';
import LinkDomainForm from '@/components/forms/LinkDomainForm.vue';
import LinkPersonForm from '@/components/forms/LinkPersonForm.vue';
import LinkShareForm from '@/components/forms/LinkShareForm.vue';

/**
 * Creates relation configs for an Application detail page.
 *
 * @param app - Reactive ref to the application with relations
 * @param appId - Application ID for API calls
 */
export function createApplicationRelationConfigs(
  app: () => ApplicationWithRelations | null,
  appId: string
): RelationConfigMap {
  return {
    infra: {
      type: 'infra',
      title: 'Infrastructure',
      singularTitle: 'Infra',
      emptyMessage: 'No infrastructure linked',
      listApi: infraApi.list,
      createApi: infraApi.create,
      linkApi: (infraId, data) =>
        applicationsApi.linkInfra(appId, infraId, data as LinkInfra),
      unlinkApi: (infraId) => applicationsApi.unlinkInfra(appId, infraId),
      createFormComponent: markRaw(InfraForm) as Component,
      linkFormComponent: markRaw(LinkInfraForm) as Component,
      hasLinkForm: true,
      canEdit: true,
      getExcludeIds: () => app()?.infra.map((i) => i.id) ?? [],
      getDisplayName: (r) => (r as InfraRelation).name,
    } as RelationConfig<InfraRelation, CreateInfra, LinkInfra>,

    service: {
      type: 'service',
      title: 'Services',
      singularTitle: 'Service',
      emptyMessage: 'No services linked',
      listApi: servicesApi.list,
      createApi: servicesApi.create,
      linkApi: (serviceId, data) =>
        applicationsApi.linkService(appId, serviceId, data as LinkService),
      unlinkApi: (serviceId) => applicationsApi.unlinkService(appId, serviceId),
      createFormComponent: markRaw(ServiceForm) as Component,
      linkFormComponent: markRaw(LinkServiceForm) as Component,
      hasLinkForm: true,
      canEdit: true,
      getExcludeIds: () => app()?.services.map((s) => s.id) ?? [],
      getDisplayName: (r) => (r as ServiceRelation).name,
    } as RelationConfig<ServiceRelation, CreateService, LinkService>,

    domain: {
      type: 'domain',
      title: 'Domains',
      singularTitle: 'Domain',
      emptyMessage: 'No domains linked',
      listApi: domainsApi.list,
      createApi: domainsApi.create,
      linkApi: (domainId, data) =>
        applicationsApi.linkDomain(appId, domainId, data as LinkDomain),
      unlinkApi: (domainId) => applicationsApi.unlinkDomain(appId, domainId),
      createFormComponent: markRaw(DomainForm) as Component,
      linkFormComponent: markRaw(LinkDomainForm) as Component,
      hasLinkForm: true,
      canEdit: true,
      getExcludeIds: () => app()?.domains.map((d) => d.id) ?? [],
      getDisplayName: (r) => (r as DomainRelation).fqdn,
    } as RelationConfig<DomainRelation, CreateDomain, LinkDomain>,

    person: {
      type: 'person',
      title: 'People',
      singularTitle: 'Person',
      emptyMessage: 'No people linked',
      listApi: peopleApi.list,
      createApi: peopleApi.create,
      linkApi: (personId, data) =>
        applicationsApi.linkPerson(appId, personId, data as LinkPerson),
      unlinkApi: (personId) => applicationsApi.unlinkPerson(appId, personId),
      createFormComponent: markRaw(PersonForm) as Component,
      linkFormComponent: markRaw(LinkPersonForm) as Component,
      hasLinkForm: true,
      canEdit: true,
      getExcludeIds: () => app()?.people.map((p) => p.id) ?? [],
      getDisplayName: (r) => (r as PersonRelation).name,
    } as RelationConfig<PersonRelation, CreatePerson, LinkPerson>,

    share: {
      type: 'share',
      title: 'Storage',
      singularTitle: 'Storage',
      emptyMessage: 'No storage linked',
      listApi: sharesApi.list,
      createApi: sharesApi.create,
      linkApi: (shareId, data) =>
        applicationsApi.linkShare(appId, shareId, data as LinkNetworkShare),
      unlinkApi: (shareId) => applicationsApi.unlinkShare(appId, shareId),
      createFormComponent: markRaw(ShareForm) as Component,
      linkFormComponent: markRaw(LinkShareForm) as Component,
      hasLinkForm: true,
      canEdit: true,
      getExcludeIds: () => app()?.network_shares.map((s) => s.id) ?? [],
      getDisplayName: (r) => (r as NetworkShareRelation).name,
    } as RelationConfig<NetworkShareRelation, CreateNetworkShare, LinkNetworkShare>,

    stack: {
      type: 'stack',
      title: 'Tech Stack',
      singularTitle: 'Stack',
      emptyMessage: 'No technologies linked',
      listApi: stacksApi.list,
      createApi: stacksApi.create,
      linkApi: (stackId) => applicationsApi.linkStack(appId, stackId),
      unlinkApi: (stackId) => applicationsApi.unlinkStack(appId, stackId),
      createFormComponent: markRaw(StackForm) as Component,
      linkFormComponent: null,
      hasLinkForm: false, // Direct link on select
      canEdit: false,
      getExcludeIds: () => app()?.stacks.map((s) => s.id) ?? [],
      getDisplayName: (r) => (r as StackRelation).name,
    } as RelationConfig<StackRelation, CreateStack, Record<string, never>>,

    health: {
      type: 'health',
      title: 'Healthchecks',
      singularTitle: 'Healthcheck',
      emptyMessage: 'No healthchecks configured',
      listApi: async () => ({ data: [], total: 0, page: 1, per_page: 0, total_pages: 0 }), // Not used
      createApi: healthchecksApi.create,
      linkApi: async () => {}, // Not used - create-only
      unlinkApi: (healthId) => healthchecksApi.delete(healthId),
      editApi: (healthId, data) =>
        healthchecksApi.update(healthId, data as UpdateHealthcheck),
      createFormComponent: markRaw(HealthcheckForm) as Component,
      linkFormComponent: null,
      hasLinkForm: false,
      createOnly: true, // No entity selection, just create
      canEdit: true,
      getExcludeIds: () => [],
      getDisplayName: (r) => (r as HealthcheckRelation).name,
      getCreateFormProps: () => ({
        initialApplicationId: appId,
        initialName: app()?.name,
        initialTargetName: app()?.name,
      }),
      fetchForEdit: async (relation: unknown) => {
        // Fetch full healthcheck data since HealthcheckRelation is missing fields
        return healthchecksApi.get((relation as HealthcheckRelation).id);
      },
    } as RelationConfig<HealthcheckRelation, CreateHealthcheck, UpdateHealthcheck>,
  };
}

/**
 * Creates relation configs for a Service detail page.
 *
 * @param service - Reactive ref to the service with relations
 * @param serviceId - Service ID for API calls
 */
export function createServiceRelationConfigs(
  service: () => ServiceWithRelations | null,
  serviceId: string
): RelationConfigMap {
  return {
    infra: {
      type: 'infra',
      title: 'Infrastructure',
      singularTitle: 'Infra',
      emptyMessage: 'No infrastructure linked',
      listApi: infraApi.list,
      createApi: infraApi.create,
      linkApi: (infraId, data) =>
        servicesApi.linkInfra(serviceId, infraId, data as LinkInfra),
      unlinkApi: (infraId) => servicesApi.unlinkInfra(serviceId, infraId),
      createFormComponent: markRaw(InfraForm) as Component,
      linkFormComponent: markRaw(LinkInfraForm) as Component,
      hasLinkForm: true,
      canEdit: true,
      getExcludeIds: () => service()?.infra.map((i) => i.id) ?? [],
      getDisplayName: (r) => (r as InfraRelation).name,
    } as RelationConfig<InfraRelation, CreateInfra, LinkInfra>,

    health: {
      type: 'health',
      title: 'Healthchecks',
      singularTitle: 'Healthcheck',
      emptyMessage: 'No healthchecks configured',
      listApi: async () => ({ data: [], total: 0, page: 1, per_page: 0, total_pages: 0 }),
      createApi: healthchecksApi.create,
      linkApi: async () => {},
      unlinkApi: (healthId) => healthchecksApi.delete(healthId),
      editApi: (healthId, data) =>
        healthchecksApi.update(healthId, data as UpdateHealthcheck),
      createFormComponent: markRaw(HealthcheckForm) as Component,
      linkFormComponent: null,
      hasLinkForm: false,
      createOnly: true,
      canEdit: true,
      getExcludeIds: () => [],
      getDisplayName: (r) => (r as HealthcheckRelation).name,
      getCreateFormProps: () => ({
        initialServiceId: serviceId,
        initialName: service()?.name,
        initialTargetName: service()?.name,
      }),
      fetchForEdit: async (relation: unknown) => {
        return healthchecksApi.get((relation as HealthcheckRelation).id);
      },
    } as RelationConfig<HealthcheckRelation, CreateHealthcheck, UpdateHealthcheck>,
  };
}
