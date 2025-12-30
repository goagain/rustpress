/**
 * URL utility functions
 */

/**
 * Convert server URL to relative path if it's from the same server
 * 
 * This function checks if a URL is from the current server (localhost:3000 or same origin),
 * and if so, converts it to a relative path. External URLs (like S3) remain unchanged.
 * 
 * @param url - The URL to convert
 * @returns Relative path if from same server, original URL otherwise
 */
export function normalizeImageUrl(url: string): string {
  if (!url) return url;
  
  // If already a relative path (starts with /), return as is
  if (url.startsWith('/')) {
    return url;
  }
  
  try {
    const urlObj = new URL(url);
    const currentOrigin = window.location.origin;
    
    // If URL is from the same origin, convert to relative path
    if (urlObj.origin === currentOrigin) {
      return urlObj.pathname + urlObj.search + urlObj.hash;
    }
    
    // External URL (like S3), return as is
    return url;
  } catch {
    // If URL parsing fails, assume it's already a relative path or invalid
    // If it doesn't start with http/https, treat as relative
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      return url.startsWith('/') ? url : `/${url}`;
    }
    
    return url;
  }
}

