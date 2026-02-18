/**
 * Constants for Admin Auto MCP Server
 */

export const CHARACTER_LIMIT = 25000;  // Maximum response size in characters

export enum ResponseFormat {
  MARKDOWN = "markdown",
  JSON = "json"
}

export enum Environment {
  PRD = "prd",
  DEV = "dev",
  STG = "stg",
  TST = "tst"
}

export enum Status {
  ACTIVE = "active",
  INACTIVE = "inactive",
  DEPRECATED = "deprecated"
}

export enum InfrastructureType {
  NOMAD = "nomad",
  KUBERNETES = "kubernetes",
  SERVER = "server",
  VM = "vm",
  CONTAINER = "container"
}

export enum ShareType {
  SMB = "smb",
  NFS = "nfs",
  WEBDAV = "webdav"
}

export enum NoteType {
  GENERAL = "general",
  INCIDENT = "incident",
  MAINTENANCE = "maintenance",
  DOCUMENTATION = "documentation"
}

export enum ContributionType {
  DEVELOPER = "developer",
  MAINTAINER = "maintainer",
  OWNER = "owner",
  CONTRIBUTOR = "contributor"
}

export enum HttpMethod {
  GET = "GET",
  POST = "POST",
  PUT = "PUT",
  DELETE = "DELETE",
  PATCH = "PATCH",
  HEAD = "HEAD"
}

export enum Protocol {
  HTTP = "http",
  HTTPS = "https"
}
