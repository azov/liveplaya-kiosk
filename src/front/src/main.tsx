import React from 'react'
import ReactDOM from 'react-dom/client'
import MapPage from './components/layouts/MapPage'
import FeatureView from './components/views/FeatureView'
import TextPage from './components/layouts/TextPage'
import LogView from './components/views/LogView'
import * as ReactRouter from 'react-router-dom';

const router = ReactRouter.createBrowserRouter([
  {
    path: "/",
    element: <Redirect to={"/map"}/>,
  },
  {
    path: "/map",
    element: <MapPage/>,
    children: [
      {
        path: "",
        element: <FeatureView/>,
      },
      {
        path: ":feature",
        element: <FeatureView/>,
      },
    ]
  },
  {
    path: "/log",
    element: <TextPage/>,
    children: [
      {
        path: "",
        element: <LogView/>,
      }
    ]
  },

]);

function Redirect(p : {to: string}) {
  let navigate = ReactRouter.useNavigate();
  React.useEffect(() => {
    navigate(p.to);
  }, [navigate]);
  return null;
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <ReactRouter.RouterProvider router={router} />
  </React.StrictMode>,
)

