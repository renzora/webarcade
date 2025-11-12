import { bridge } from '@/api/bridge';

export async function fetchHello() {
  const response = await bridge('/test/hello');
  return response.json();
}
