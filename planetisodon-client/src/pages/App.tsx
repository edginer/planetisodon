import { Outlet, Link } from "react-router-dom";

function App() {
  let outlet = Outlet({});
  if (outlet == null) {
    outlet = <div>Not Found Outlet</div>;
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
