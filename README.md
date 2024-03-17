# Planetisodon

Experimental anonymous BBS system on Cloudflare workers

## 概要

- 書き込みにGoogle認証必須 
  - メールアドレス等はサーバ上で保持しない
- バックエンドに[Planetscale](https://planetscale.com/)を使用 (NeonかTiDB Serverlessに移行予定)
- スレタイにスレ立て者のIDを付与
  - MateとWeb版限定、Headerで"X-ThreadList-AuthorId-Supported: true"にすれば取得可能
- Web版の改善
- etc

## Demo

https://planetisodon.eddibb.cc/planetisodon/
