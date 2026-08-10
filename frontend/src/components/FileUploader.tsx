import { ChangeEvent, useRef } from "react";
import { Button } from "./ui";

/** Single-file picker; returns the raw File for the workforce upload flow. */
export function FileUploader({
  accept,
  onSelect,
  label = "Choose file",
}: {
  accept?: string;
  onSelect: (file: File) => void;
  label?: string;
}) {
  const inputRef = useRef<HTMLInputElement>(null);
  const handle = (e: ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) onSelect(file);
    e.target.value = "";
  };
  return (
    <>
      <input
        ref={inputRef}
        type="file"
        accept={accept}
        className="hidden"
        onChange={handle}
        data-testid="file-input"
      />
      <Button type="button" variant="secondary" onClick={() => inputRef.current?.click()}>
        {label}
      </Button>
    </>
  );
}
