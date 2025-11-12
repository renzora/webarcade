import { bridge } from '@/api/bridge';

export async function fetchHello() {
  const response = await bridge('/demo/hello');
  return response.json();
}
