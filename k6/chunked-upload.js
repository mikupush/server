import http from 'k6/http';
import { check } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

const BASE_URL = __ENV.BASE_URL ?? 'http://localhost:8080'

export const options = {
  stages: [
    { duration: '1m', target: 100 },
    { duration: '1m', target: 500 },
    { duration: '1m', target: 0 },
  ],
}

const FILE_SIZE = 1024 * 1024 * 100 // 100MB
const FILE_CHUNK_SIZE = 1024 * 1024 * 10 // Should be 10MB max

// Pre-allocate buffer once to save memory (approx 10MB RAM total for all VUs to share logic, though k6 copies data per VU)
const FIXED_CHUNK_BUFFER = new Uint8Array(FILE_CHUNK_SIZE).fill(120).buffer

export default function() {
  const file = {
    id: uuidv4(),
    name: "example_file.txt",
    mime_type: "text/plain",
    size: FILE_SIZE
  }

  const registerResponse = http.post(`${BASE_URL}/api/file`, JSON.stringify(file), {
    headers: {
      'Content-Type': 'application/json',
    }
  })

  check(registerResponse, {
    'is status 200': (r) => r.status === 200,
  })

  const parts = file.size / FILE_CHUNK_SIZE;

  for (let i = 0; i < parts; i++) {
    const uploadResponse =http.post(`${BASE_URL}/api/file/${file.id}/upload/part/${i}`, FIXED_CHUNK_BUFFER, {
      headers: {
        'Content-Type': 'application/octet-stream',
      }
    })

    check(uploadResponse, {
      'is status 200': (r) => r.status === 200,
    })
  }
}
