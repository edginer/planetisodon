import { Outlet, useNavigate, useParams } from "react-router-dom";
import { useSuspenseQuery } from "@tanstack/react-query";

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
  return `${date.getFullYear()}/${date.getMonth()}/${date.getDay()} ${date.getHours()}:${date.getMinutes()}`;
};

const ThreadListView = () => {
  let outlet = Outlet({});
  if (outlet == null) {
    outlet = <div />;
  }
  const params = useParams();
  const navigate = useNavigate();

  const { data } = useSuspenseQuery({
    queryKey: ["threadList", params.boardKey],
    queryFn: async () => {
      const res = await fetch(`/${params.boardKey}/subject.txt`, {
        headers: {
          "Content-Type": "text/plain; charset=shift_jis",
          "X-ThreadList-AuthorId-Supported": "true",
        },
      });
      const sjisText = await res.blob();
      const arrayBuffer = await sjisText.arrayBuffer();
      const text = new TextDecoder("shift_jis").decode(arrayBuffer);

      return convertSubjectTextToThreadList(text);
    },
  });

  return (
    <>
      <div className="sm:w-96">
        <div className="divide-y-2 divide-gray-700 flex flex-col">
          {data.map((thread) => (
            <button
              key={thread.id}
              className="hover:bg-gray-200 cursor-default text-left block"
              onClick={() => {
                navigate(`/${params.boardKey}/${thread.id}`);
              }}
            >
              <div>
                <span>{thread.title}</span>
                <span> ({thread.responseCount})</span>
              </div>
              <div>
                <span>{convertLinuxTimeToDateString(thread.id)}</span>
                <span> ID:{thread.authorId}</span>
              </div>
            </button>
          ))}
        </div>
      </div>
      {outlet}
    </>
  );
};

export default ThreadListView;
