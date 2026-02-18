#!/usr/bin/env node
/**
 * Debug script to test Admin Auto API connection
 */
import dotenv from 'dotenv';
import axios from 'axios';

dotenv.config();

const AUTO_URL = process.env.AUTO_URL;
const AUTO_USERNAME = process.env.AUTO_USERNAME;
const AUTO_PASSWORD = process.env.AUTO_PASSWORD;

console.log('🔍 Testing Admin Auto API Connection\n');
console.log('Configuration:');
console.log('  URL:', AUTO_URL);
console.log('  Username:', AUTO_USERNAME);
console.log('  Password:', AUTO_PASSWORD ? '***' + AUTO_PASSWORD.slice(-3) : 'NOT SET');
console.log('');

if (!AUTO_URL || !AUTO_USERNAME || !AUTO_PASSWORD) {
  console.error('❌ Missing credentials in .env file');
  process.exit(1);
}

// Test 1: Health check without auth
console.log('Test 1: Health check (no auth required)');
try {
  const response = await axios.get(`${AUTO_URL}/health`, { timeout: 10000 });
  console.log('✅ Health check successful:', response.data);
} catch (error) {
  console.log('❌ Health check failed:', error.message);
  if (error.code === 'ENOTFOUND' || error.code === 'ECONNREFUSED') {
    console.log('   Cannot reach server. Check URL and network connection.');
    process.exit(1);
  }
}
console.log('');

// Test 2: API endpoint with Basic Auth
console.log('Test 2: API endpoint with HTTP Basic Auth');
try {
  const response = await axios.get(`${AUTO_URL}/api/dashboard/stats`, {
    timeout: 10000,
    auth: {
      username: AUTO_USERNAME,
      password: AUTO_PASSWORD
    },
    headers: {
      'Accept': 'application/json',
      'Content-Type': 'application/json'
    }
  });
  console.log('✅ Authentication successful!');
  console.log('   Response:', JSON.stringify(response.data, null, 2));
} catch (error) {
  console.log('❌ Authentication failed');
  console.log('   Status:', error.response?.status);
  console.log('   Message:', error.response?.statusText);
  console.log('   Data:', error.response?.data);
  console.log('');

  if (error.response?.status === 401) {
    console.log('💡 Debug Info:');
    console.log('   - Verify username and password are correct');
    console.log('   - Check if credentials have special characters that need escaping');
    console.log('   - Verify the Admin Auto instance requires Basic Auth');
    console.log('');
    console.log('🧪 Testing auth header format:');
    const authString = `${AUTO_USERNAME}:${AUTO_PASSWORD}`;
    const base64Auth = Buffer.from(authString).toString('base64');
    console.log('   Auth string:', `${AUTO_USERNAME}:***`);
    console.log('   Base64 encoded:', base64Auth);
  }

  if (error.response?.status === 403) {
    console.log('💡 Permission denied - credentials may be correct but lack permissions');
  }
}
console.log('');

// Test 3: Try with explicit Authorization header
console.log('Test 3: Trying with explicit Authorization header');
try {
  const authString = `${AUTO_USERNAME}:${AUTO_PASSWORD}`;
  const base64Auth = Buffer.from(authString).toString('base64');

  const response = await axios.get(`${AUTO_URL}/api/dashboard/stats`, {
    timeout: 10000,
    headers: {
      'Authorization': `Basic ${base64Auth}`,
      'Accept': 'application/json',
      'Content-Type': 'application/json'
    }
  });
  console.log('✅ Explicit Authorization header successful!');
  console.log('   Response:', JSON.stringify(response.data, null, 2));
} catch (error) {
  console.log('❌ Explicit Authorization header failed');
  console.log('   Status:', error.response?.status);
  console.log('   Message:', error.response?.statusText);
}
