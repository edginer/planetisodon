import { useEffect, useState } from "react";
import { Outlet, useParams, Link } from "react-router-dom";

interface Thread {
  title: string;
  id: number;
  responseCount: number;
  authorId: string;
}

const convertSubjectTextToThreadList = (text: string): Thread[] => {
  const lines = text.split("\n");
  const threadList = lines
    .map((line) => {
      const lineRegex = /^(\d{9,10}\.dat)<>(.*) \[(.{4,13})★\] \((\d{1,5})\)$/;
      const match = line.match(lineRegex);
      if (match == null) {
        return undefined;
      }
      const id = parseInt(match[1].split(".")[0]);
      const title = match[2];
      const authorId = match[3];
      const responseCount = parseInt(match[4]);

      return {
        title,
        id,
        responseCount,
        authorId,
      };
    })
    .filter((thread) => thread != null) as Thread[];
  return threadList;
};

const convertLinuxTimeToDateString = (linuxTime: number): string => {
  const date = new Date(linuxTime * 1000);
  return date.toISOString();
};

const ThreadListView = () => {
  const params = useParams();
  const [threadList, setThreadList] = useState<Thread[]>([]);
  useEffect(() => {
    const f = async () => {
      const res = await fetch(`/${params.boardKey}/subject.txt`, {
        headers: {
          "Content-Type": "text/plain; charset=shift_jis",
          "X-Request-From-Planetisodon-Client": "true",
        },
      });
      const sjisText = await res.blob();
      const arrayBuffer = await sjisText.arrayBuffer();
      const text = new TextDecoder("shift_jis").decode(arrayBuffer);
      const newThreadList = convertSubjectTextToThreadList(text);

      setThreadList(newThreadList);
    };
    f();
  }, [params.boardKey, setThreadList]);

  return (
    <>
      <div className="sm:w-96">
        <div className="">
          {threadList.map((thread) => (
            <div key={thread.id}>
              <Link to={`/${params.boardKey}/${thread.id}`}>
                {thread.title}
              </Link>
              <span>({thread.responseCount})</span>
              <span>{thread.authorId}</span>
              <span>{convertLinuxTimeToDateString(thread.id)}</span>
            </div>
          ))}
        </div>
      </div>
      <Outlet />
    </>
  );
};

export default ThreadListView;
