/** @type {import('next').NextConfig} */
const apiInternalUrl = (process.env.API_INTERNAL_URL || 'http://127.0.0.1:8180').replace(/\/$/, '');

const nextConfig = {
  // Bản dựng tự chứa cho Docker (node server.js), không cần node_modules đầy đủ lúc chạy.
  output: 'standalone',
  poweredByHeader: false,
  reactStrictMode: true,
  images: {
    formats: ['image/avif', 'image/webp'],
    deviceSizes: [390, 640, 828, 1080, 1280, 1536, 1920, 2172],
    minimumCacheTTL: 60 * 60 * 24 * 30,
  },
  // Trình duyệt luôn gọi /api cùng origin; Next chuyển tiếp sang backend Rust.
  // Ở production Nginx định tuyến /api thẳng tới backend nên rewrite này chỉ còn là dự phòng.
  async rewrites() {
    return [{ source: '/api/:path*', destination: `${apiInternalUrl}/api/:path*` }];
  },
  async headers() {
    return [
      {
        source: '/images/:path*',
        headers: [{ key: 'Cache-Control', value: 'public, max-age=2592000, stale-while-revalidate=86400' }],
      },
    ];
  },
};

export default nextConfig;
