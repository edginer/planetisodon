import { Outlet, Link } from "react-router-dom";

function App() {
  let outlet = Outlet({});
  if (outlet == null) {
    outlet = (
      <div>
        <h1 className="text-4xl p-2">Plantisodon</h1>
        <h4 className="text-xl pl-3">本掲示板の特徴</h4>
        <ul className="list-inside list-disc pl-8">
          <li>
            書き込みにGoogle認証必須 (メールアドレス等はサーバ上で保持しない)
          </li>
          <li>
            バックエンドに
            <a
              className="underline hover:text-red-400"
              href="https://planetscale.com/"
              rel="noreferrer"
            >
              Planetscale
            </a>
            を使用 (NeonかTiDB Serverlessに移行予定)
          </li>
          <li>
            スレタイにスレ立て者のIDを付与(MateとWeb版限定、Headerで"X-ThreadList-AuthorId-Supported:
            true"にすれば取得可能)
          </li>
          <li>Web版の改善</li>
          <li>etc</li>
        </ul>
        <h4 className="text-xl pl-3">認証ページ</h4>
        <div className="pl-6 ">
          <p className="underline hover:text-red-400">
            <a href="/auth" rel="noreferrer">
              こちら
            </a>
          </p>
          <p>
            {" "}
            アプリ内ブラウザからログインできない場合があるため、Chromeなどの外部ブラウザでログインしてください
          </p>
          <p>↓コピペ用リンク</p>
          <p>https://planetisodon.eddibb.cc/auth</p>
        </div>
        <h4 className="text-xl pl-3">その他</h4>
        <a
          className="pl-6 underline hover:text-red-400"
          href="https://bbs.eddibb.cc/"
          rel="noreferrer"
        >
          こちらへ
        </a>
      </div>
    );
  }

  return (
    <div className="flex flex-col sm:flex-row sm:divide-x-2 sm:h-screen">
      <div className="sm:flex sm:flex-col sm:w-48 sm:divide-y-2">
        <span className="p-1">Board List</span>
        <div className="p-1">
          <ul>
            <Link to="/planetisodon/">Planetisodon</Link>
          </ul>
        </div>
      </div>
      {outlet}
    </div>
  );
}

export default App;
