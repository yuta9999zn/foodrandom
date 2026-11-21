"""Thử tải đơn giản cho FoodRandom (chỉ dùng thư viện chuẩn của Python).

    python tools/loadtest.py http://localhost:8090 --concurrency 200 --seconds 30

Mô phỏng người dùng: xem danh sách món theo bộ lọc ngẫu nhiên, đổi trang, xem chi tiết, bấm random.
In ra số request, tỉ lệ lỗi và độ trễ p50/p95/p99.
"""

import argparse
import json
import random
import socket
import statistics
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
from collections import Counter

MARKETS = ['VN', 'JP', 'KR', 'CN', 'GB']
LANGS = ['vi', 'en', 'ja', 'ko', 'zh']
MEALS = ['breakfast', 'lunch', 'dinner', 'snack']
CUISINES = ['vietnam', 'japan', 'china', 'korea', 'uk']
SLUGS = ['pho-bo', 'ramen-tonkotsu', 'com-tron-bibimbap', 'fish-and-chips', 'vit-quay-bac-kinh', 'banh-mi-thit', 'sushi-ca-hoi']


BASES = []


def resolve_bases(base):
    """Tra DNS một lần và rải request đều sang mọi bản sao (tránh đo nhầm tốc độ DNS thay vì ứng dụng)."""
    parsed = urllib.parse.urlsplit(base)
    port = parsed.port or 80
    ips = sorted({info[4][0] for info in socket.getaddrinfo(parsed.hostname, port, socket.AF_INET)})
    return [f'{parsed.scheme}://{ip}:{port}' for ip in ips]


def request(base, method, path, body=None):
    base = random.choice(BASES) if BASES else base
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(base + path, data=data, method=method, headers={'Content-Type': 'application/json'})
    start = time.perf_counter()
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            resp.read()
            status = resp.status
    except urllib.error.HTTPError as err:
        status = err.code
    except Exception:
        status = 0
    return status, (time.perf_counter() - start) * 1000


def scenario(base):
    market, lang = random.choice(MARKETS), random.choice(LANGS)
    query = f'market={market}&lang={lang}&meals={random.choice(MEALS)}&page={random.randint(1, 2)}'
    if random.random() < 0.5:
        query += f'&cuisines={random.choice(CUISINES)}'
    roll = random.random()
    if roll < 0.55:
        return 'list', request(base, 'GET', f'/api/v1/foods?{query}')
    if roll < 0.80:
        return 'detail', request(base, 'GET', f'/api/v1/foods/{random.choice(SLUGS)}?market={market}&lang={lang}')
    body = {'filter': {'market': market, 'lang': lang, 'meals': [random.choice(MEALS)]}, 'page': 1, 'scope': 'page'}
    return 'random', request(base, 'POST', '/api/v1/random', body)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('base')
    parser.add_argument('--concurrency', type=int, default=100)
    parser.add_argument('--seconds', type=int, default=20)
    args = parser.parse_args()
    BASES.extend(resolve_bases(args.base))
    print(f'Đích: {BASES}')

    deadline = time.time() + args.seconds
    lock = threading.Lock()
    latencies, statuses, kinds = [], Counter(), Counter()

    def worker():
        while time.time() < deadline:
            kind, (status, ms) = scenario(args.base)
            with lock:
                latencies.append(ms)
                statuses[status] += 1
                kinds[kind] += 1

    threads = [threading.Thread(target=worker, daemon=True) for _ in range(args.concurrency)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    total = len(latencies)
    ok = sum(c for s, c in statuses.items() if 200 <= s < 300)
    q = statistics.quantiles(latencies, n=100) if total > 1 else [0] * 99
    print(f'{args.concurrency} người dùng đồng thời trong {args.seconds} giây')
    print(f'  Tổng request : {total}  (~{total / args.seconds:.0f} req/s)  {dict(kinds)}')
    print(f'  Thành công   : {ok} ({ok / max(total, 1):.1%})   mã trả về: {dict(statuses)}')
    print(f'  Độ trễ (ms)  : p50={q[49]:.0f}  p95={q[94]:.0f}  p99={q[98]:.0f}  max={max(latencies):.0f}')


if __name__ == '__main__':
    main()
