"""Tell IndexNow search engines (Bing, Yandex and others) that the site's pages changed.

Run after deploying dist/site: posts every URL in its sitemap.xml with the key from
site/site.json, whose key file the build places at the site root. Standard library only.
"""
import json
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
config = json.loads((ROOT / "site" / "site.json").read_text(encoding="utf-8"))
sitemap = (ROOT / "dist" / "site" / "sitemap.xml").read_text(encoding="utf-8")
urls = re.findall(r"<loc>(.*?)</loc>", sitemap)
key = config["indexnow_key"]
host = config["domain"]
body = json.dumps({"host": host, "key": key, "keyLocation": f"https://{host}/{key}.txt", "urlList": urls}).encode()
request = urllib.request.Request("https://api.indexnow.org/indexnow", data=body,
                                 headers={"Content-Type": "application/json; charset=utf-8"})
try:
    with urllib.request.urlopen(request, timeout=30) as response:
        print(f"IndexNow: {response.status} for {len(urls)} URLs")
except urllib.error.HTTPError as error:
    sys.exit(f"IndexNow: {error.code} {error.read().decode(errors='replace')[:200]}")
