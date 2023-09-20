const CACHE_NAME = 'my-pwa-cache-v1';
const urlsToCache = [
    '/',
    'index.html',
    'manifest.json',
    'app.js',
    'main.css',
    'static/android-chrome-144x144.png',
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then((cache) => {
        return cache.addAll(urlsToCache);
      })
  );
});

self.addEventListener('fetch', (event) => {
    if (event.request.url.startsWith('chrome-extension://')) {
      return; // Skip caching requests from Chrome extensions
    }
  
    event.respondWith(
      caches.match(event.request)
        .then((response) => {
          if (response) {
            return response; // Cache hit - return the response from the cache
          }
  
          // Clone the request because it can only be consumed once
          const fetchRequest = event.request.clone();
  
          return fetch(fetchRequest)
            .then((response) => {
              if (!response || response.status !== 200 || response.type !== 'basic') {
                return response;
              }
  
              // Clone the response to cache it
              const responseToCache = response.clone();
  
              caches.open(CACHE_NAME)
                .then((cache) => {
                  cache.put(event.request, responseToCache);
                });
  
              return response;
            });
        })
        .catch(() => {
          // Handle errors, e.g., if the fetch request fails
          // You can add custom error handling here
        })
    );
});  

self.addEventListener('activate', (event) => {
  const cacheWhitelist = [CACHE_NAME];

  event.waitUntil(
    caches.keys()
      .then((cacheNames) => {
        return Promise.all(
          cacheNames.map((cacheName) => {
            if (cacheWhitelist.indexOf(cacheName) === -1) {
              return caches.delete(cacheName);
            }
            return null;
          })
        );
      })
  );
});
