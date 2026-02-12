import {useState} from "react";
import {Button} from "@/components/ui/button.tsx";

export default function Counter() {
  const [count, setCount] = useState(0);

  return (
    <Button onClick={() => setCount(count + 1)}>Count {count}</Button>
  )
}
