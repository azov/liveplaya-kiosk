import largeLogoUrl from '../../assets/logo-l.png';

interface Props {
    size: "small"|"large",
}

const WIDTH_PX={
    small: 100,
    large: 300,
}

export default function Logo(p : Props = {size: "small"}) {
    return (
        <img src={largeLogoUrl} width={WIDTH_PX[p.size]}/>
    );
}
