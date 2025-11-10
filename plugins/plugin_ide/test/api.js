export async function fetchHello() {
  const response = await fetch('/test/hello');
  return response.json();
}
