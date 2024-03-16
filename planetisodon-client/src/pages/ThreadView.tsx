import { useState } from "react";
import { useParams } from "react-router-dom";
import { useSuspenseQuery } from "@tanstack/react-query";
import Encoding from "encoding-japanese";

interface Response {
  name: string;
  mail: string;
  date: string;
  authorId: string;
  body: string;
  id: number;
}

const convertThreadTextToResponseList = (
  text: string
): [string, Response[]] => {
  const lines = text.split("\n").filter((x) => x !== "");
  let threadTitle = "";
  const responses = lines.map((line, idx) => {
    const lineRegex = /^(.*)<>(.*)<>(.*) ID:(.*)<>(.*)<>(.*)$/;
    const match = line.match(lineRegex);
    if (match == null) {
      throw new Error(`Invalid response line: ${line}`);
    }
    const name = match[1];
    const mail = match[2];
    const date = match[3];
    const authorId = match[4];
    const body = match[5];
    if (idx === 0) {
      threadTitle = match[6];
    }

    return {
      name,
      mail,
      date,
      authorId,
      body,
      id: idx + 1,
    };
  });

  return [threadTitle, responses satisfies Response[]];
};

const convertToSjisText = (text: string): string => {
  const sjis = Encoding.convert(Encoding.stringToCode(text), {
    to: "SJIS",
    from: "UNICODE",
  });
  return Encoding.urlEncode(sjis);
};

const postResponse = async (
  boardKey: string,
  threadKey: string,
  name: string,
  mail: string,
  body: string
) => {
  const params = {
    submit: convertToSjisText("書き込む"),
    mail: convertToSjisText(mail),
    FROM: convertToSjisText(name),
    MESSAGE: convertToSjisText(body),
    bbs: boardKey,
    key: threadKey,
  };

  const res = await fetch(`/test/bbs.cgi`, {
    method: "POST",
    headers: {
      "Content-Type": "application/x-www-form-urlencoded",
    },
    body:
      "submit=" +
      params.submit +
      "&mail=" +
      params.mail +
      "&FROM=" +
      params.FROM +
      "&MESSAGE=" +
      params.MESSAGE +
      "&bbs=" +
      params.bbs +
      "&key=" +
      params.key,
  });
  if (!res.ok) {
    throw new Error(`Failed to post a response: ${res.statusText}`);
  }
};

const ThreadView = () => {
  const params = useParams();
  const [body, setBody] = useState("");
  const [name, setName] = useState("");
  const [mail, setMail] = useState("");

  const { data } = useSuspenseQuery({
    queryKey: ["thread", params.boardKey, params.threadKey],
    queryFn: async () => {
      const res = await fetch(
        `/${params.boardKey}/dat/${params.threadKey}.dat`,
        {
          headers: {
            "Content-Type": "text/plain; charset=shift_jis",
          },
        }
      );
      const sjisText = await res.blob();
      const arrayBuffer = await sjisText.arrayBuffer();
      const text = new TextDecoder("shift_jis").decode(arrayBuffer);
      return convertThreadTextToResponseList(text);
    },
  });

  return (
    <div>
      <h2>{data[0]}</h2>
      <div className="flex">
        <ul>
          {data[1].map((response) => (
            <li key={response.id}>
              <div>{response.name}</div>
              <div>{response.date}</div>
              <div dangerouslySetInnerHTML={{ __html: response.body }} />
            </li>
          ))}
        </ul>
      </div>
      <div className="flex-grow">
        <div className="flex flex-col">
          <input
            type="text"
            placeholder="Name"
            onChange={(e) => setName(e.target.value)}
          />
          <input
            type="text"
            placeholder="Mail"
            onChange={(e) => setMail(e.target.value)}
          />
        </div>
        <textarea
          className="w-full"
          placeholder="Post a response"
          onChange={(e) => setBody(e.target.value)}
        ></textarea>
        <button
          onClick={() =>
            postResponse(params.boardKey!, params.threadKey!, name, mail, body)
          }
        >
          Post
        </button>
      </div>
    </div>
  );
};

export default ThreadView;
