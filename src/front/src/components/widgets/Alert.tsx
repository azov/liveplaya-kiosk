interface Props {
    children: any,
    level: "info" | "error" | "success",
}

const COLOR = {
    error: '#ff0000',
    success: '#00ff00',
    info: '#cccccc',
}

export default function Alert(p: Props) {
    return <div className={`alert ${p.level}`} style={{ backgroundColor: COLOR[p.level] }}>{p.children}</div>
}
