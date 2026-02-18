/**
 * API Client for Admin Auto instance with HTTP Basic Authentication
 */

import axios, { AxiosError, AxiosInstance } from "axios";

export class AutoApiClient {
  private client: AxiosInstance;
  private baseUrl: string;

  constructor(baseUrl: string, username: string, password: string) {
    this.baseUrl = baseUrl.replace(/\/$/, ""); // Remove trailing slash

    this.client = axios.create({
      baseURL: `${this.baseUrl}/api`,
      timeout: 30000,
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
      },
      auth: {
        username,
        password,
      },
    });
  }

  /**
   * Make a GET request to the API
   */
  async get<T>(endpoint: string, params?: Record<string, any>): Promise<T> {
    try {
      const response = await this.client.get<T>(endpoint, { params });
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Make a POST request to the API
   */
  async post<T>(endpoint: string, data?: any): Promise<T> {
    try {
      const response = await this.client.post<T>(endpoint, data);
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Make a PUT request to the API
   */
  async put<T>(endpoint: string, data?: any): Promise<T> {
    try {
      const response = await this.client.put<T>(endpoint, data);
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Make a DELETE request to the API
   */
  async delete<T>(endpoint: string): Promise<T> {
    try {
      const response = await this.client.delete<T>(endpoint);
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Handle API errors and convert to user-friendly messages
   */
  private handleError(error: unknown): Error {
    if (error instanceof AxiosError) {
      if (error.response) {
        const status = error.response.status;
        const message =
          error.response.data?.message ||
          error.response.data?.error ||
          error.response.statusText;

        switch (status) {
          case 400:
            return new Error(
              `Bad Request: ${message}. Please check your input parameters.`,
            );
          case 401:
            return new Error(
              `Authentication failed: ${message}. Please check your username and password.`,
            );
          case 403:
            return new Error(
              `Permission denied: ${message}. You don't have access to this resource.`,
            );
          case 404:
            return new Error(
              `Resource not found: ${message}. Please check the ID is correct.`,
            );
          case 409:
            return new Error(
              `Conflict: ${message}. The resource may already exist.`,
            );
          case 422:
            return new Error(
              `Validation error: ${message}. Please check your input data.`,
            );
          case 429:
            return new Error(
              `Rate limit exceeded: ${message}. Please wait before making more requests.`,
            );
          case 500:
            return new Error(
              `Server error: ${message}. Please try again later or contact support.`,
            );
          case 503:
            return new Error(
              `Service unavailable: ${message}. The server may be temporarily down.`,
            );
          default:
            return new Error(`API request failed (${status}): ${message}`);
        }
      } else if (error.code === "ECONNABORTED") {
        return new Error(
          "Request timed out. Please try again or check your connection.",
        );
      } else if (error.code === "ENOTFOUND" || error.code === "ECONNREFUSED") {
        return new Error(
          `Cannot connect to Admin Auto instance at ${this.baseUrl}. Please check the URL and your network connection.`,
        );
      } else if (error.code === "ERR_NETWORK") {
        return new Error(
          `Network error: Cannot reach ${this.baseUrl}. Please check your connection.`,
        );
      }
    }

    return new Error(
      `Unexpected error: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}
