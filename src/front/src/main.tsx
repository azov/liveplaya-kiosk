import React from 'react'
import ReactDOM from 'react-dom/client'
import FullPage from './components/layouts/FullPage'
import TextPage from './components/layouts/TextPage'
import MapView from './components/views/MapView'
import LogView from './components/views/LogView'
import ListView from './components/views/ListView'
import * as ReactRouter from 'react-router-dom';

const router = ReactRouter.createHashRouter([
  {
    path: "/",
    element: <Redirect to={"/map"}/>,
  },
  {
    path: "/map",
    element: <FullPage/>,
    children: [
      {
        path: "",
        element: <MapView/>,
      },
    ]
  },
  {
    path: "/list",
    element: <TextPage/>,
    children: [
      {
        path: "",
        element: <ListView/>,
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

