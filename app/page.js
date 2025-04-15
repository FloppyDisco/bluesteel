import Image from "next/image";

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-between">
      <Image
        src="/images/background.jpeg"
        alt="Background"
        width={1024}
        height={768}
        // fill={true}
        quality={100}
      />
    </main>
  );
}
